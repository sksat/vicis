use super::super::function;
use super::Module;
use super::{
    attributes::{parser::parse_attributes, Attribute},
    global_variable, name,
    name::{parser::parse as parse_name, Name},
    Meta,
};
use crate::ir::{
    types,
    util::{spaces, string_literal},
    value::parser::parse_constant_intv,
};
use nom;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1},
    error::VerboseError,
    multi::separated_list0,
    sequence::{preceded, separated_pair, tuple},
    IResult,
};

fn parse_source_filename(source: &str) -> IResult<&str, String, VerboseError<&str>> {
    tuple((
        tag("source_filename"),
        preceded(spaces, char('=')),
        preceded(spaces, string_literal),
    ))(source)
    .map(|(i, (_, _, name))| (i, name))
}

fn parse_target_datalayout(source: &str) -> IResult<&str, String, VerboseError<&str>> {
    tuple((
        tag("target"),
        preceded(spaces, tag("datalayout")),
        preceded(spaces, char('=')),
        preceded(spaces, string_literal),
    ))(source)
    .map(|(i, (_, _, _, datalayout))| (i, datalayout))
}

fn parse_target_triple(source: &str) -> IResult<&str, String, VerboseError<&str>> {
    tuple((
        tag("target"),
        preceded(spaces, tag("triple")),
        preceded(spaces, char('=')),
        preceded(spaces, string_literal),
    ))(source)
    .map(|(i, (_, _, _, triple))| (i, triple))
}

fn parse_attribute_group(source: &str) -> IResult<&str, (u32, Vec<Attribute>), VerboseError<&str>> {
    tuple((
        tag("attributes"),
        preceded(spaces, char('#')),
        digit1,
        preceded(spaces, char('=')),
        preceded(spaces, char('{')),
        preceded(spaces, parse_attributes),
        preceded(spaces, char('}')),
    ))(source)
    .map(|(i, (_, _, id, _, _, attrs, _))| (i, (id.parse().unwrap(), attrs)))
}

fn parse_metadata(
    types: &types::Types,
) -> impl Fn(&str) -> IResult<&str, (Name, Meta), VerboseError<&str>> + '_ {
    fn parse_op(s: &str) -> IResult<&str, &str, VerboseError<&str>> {
        preceded(spaces, tag("!"))(s)
    }
    fn parse_mname(s: &str) -> IResult<&str, Name, VerboseError<&str>> {
        preceded(parse_op, parse_name)(s)
    }
    fn meta_string(s: &str) -> IResult<&str, Meta, VerboseError<&str>> {
        preceded(parse_op, preceded(spaces, string_literal))(s).map(|(i, s)| (i, Meta::String(s)))
    }
    fn meta_name(s: &str) -> IResult<&str, Meta, VerboseError<&str>> {
        parse_mname(s).map(|(i, s)| (i, Meta::Name(s)))
    }
    fn parse_type(
        types: &types::Types,
    ) -> impl Fn(&str) -> IResult<&str, types::TypeId, VerboseError<&str>> + '_ {
        move |s| types::parse(s, &types)
    }
    fn meta_int(
        types: &types::Types,
    ) -> impl Fn(&str) -> IResult<&str, Meta, VerboseError<&str>> + '_ {
        move |s| {
            let (s, (ty, _)) = tuple((parse_type(types), spaces))(s)?;
            parse_constant_intv(s, &types, ty).map(|(s, v)| (s, Meta::Int(v)))
        }
    }
    fn meta_metas(
        types: &types::Types,
    ) -> impl Fn(&str) -> IResult<&str, Meta, VerboseError<&str>> + '_ {
        move |s| {
            tuple((
                parse_op,
                spaces,
                char('{'),
                separated_list0(preceded(spaces, tag(",")), meta(types)),
                spaces,
                char('}'),
            ))(s)
            .map(|(i, (_, _, _, r, _, _))| (i, Meta::Metas(r)))
        }
    }
    fn meta(
        types: &types::Types,
    ) -> impl Fn(&str) -> IResult<&str, Meta, VerboseError<&str>> + '_ {
        move |s| alt((meta_string, meta_name, meta_metas(types), meta_int(types)))(s)
    }

    move |s| separated_pair(parse_mname, preceded(spaces, tag("=")), meta(types))(s)
}

fn parse_local_type<'a>(
    source: &'a str,
    types: &types::Types,
) -> IResult<&'a str, (), VerboseError<&'a str>> {
    let (source, name) = preceded(spaces, preceded(char('%'), name::parse))(source)?;
    types.base_mut().named_type(name.clone()); // register a named type
    let (source, _) = preceded(spaces, preceded(char('='), preceded(spaces, tag("type"))))(source)?;
    let (source, ty) = types::parse(source, types)?;
    types.base_mut().change_to_named_type(ty, name);
    Ok((source, ()))
}

pub fn parse(mut source: &str) -> Result<Module, nom::Err<VerboseError<&str>>> {
    let mut module = Module::new();
    loop {
        source = spaces(source)?.0;

        if source.is_empty() {
            break;
        }

        if let Ok((source_, source_filename)) = parse_source_filename(source) {
            module.source_filename = source_filename;
            source = source_;
            continue;
        }

        if let Ok((source_, target_datalayout)) = parse_target_datalayout(source) {
            module.target.datalayout = target_datalayout.to_string();
            source = source_;
            continue;
        }

        if let Ok((source_, target_triple)) = parse_target_triple(source) {
            module.target.triple = target_triple.to_string();
            source = source_;
            continue;
        }

        if let Ok((source_, (id, attrs))) = parse_attribute_group(source) {
            module.attributes.insert(id, attrs);
            source = source_;
            continue;
        }

        if let Ok((source_, _)) = parse_local_type(source, &module.types) {
            source = source_;
            continue;
        }

        if let Ok((source_, gv)) = global_variable::parse(source, &module.types) {
            module.global_variables.insert(gv.name.clone(), gv);
            source = source_;
            continue;
        }

        if let Ok((source_, func)) = function::parse(source, module.types.clone()) {
            module.functions.alloc(func);
            source = source_;
            continue;
        }
        if let Ok((source_, (name_, meta))) = parse_metadata(&module.types)(source) {
            module.metas.insert(name_, meta);
            source = source_;
            continue;
        }

        todo!("unsupported syntax at {:?}", source)
    }

    Ok(module)
}

#[test]
fn parse_all_examples() {
    use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
    use nom::error::convert_error;
    use std::{fs, io::Write, process};

    let files_count = fs::read_dir("./examples")
        .expect("Failed to open file")
        .count() as u64;
    let paths = fs::read_dir("./examples").unwrap();
    let pb = ProgressBar::with_draw_target(files_count, ProgressDrawTarget::stdout());
    pb.set_style(ProgressStyle::default_bar().template("{bar:60} {pos:>4}/{len:>4} {msg}"));
    for path in paths {
        let name = path.as_ref().unwrap().path().to_str().unwrap().to_string();
        pb.set_message(name.as_str());

        let source = fs::read_to_string(name).unwrap();
        let module = match parse(&source) {
            Ok(ok) => ok,
            Err(nom::Err::Error(e)) => {
                println!("{}", convert_error(source.as_str(), e));
                panic!()
            }
            Err(e) => panic!("{:?}", e),
        };
        // crate::ir::pass::dce::run_on_module(&mut module);

        {
            let mut file = fs::File::create("/tmp/output.ll").unwrap();
            write!(file, "{:?}", module).unwrap();
            file.flush().unwrap();
        }
        assert!(process::Command::new("llc-12")
            .args(&["/tmp/output.ll"])
            .stderr(process::Stdio::null())
            .status()
            .unwrap()
            .success());
        pb.inc(1);
    }
    pb.finish();
}

#[test]
fn parse_module1() {
    let result = parse(
        r#"
            source_filename = "c.c"
            target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
            ; comments
            ; bluh bluh
            target triple = "x86_64-pc-linux-gnu" ; hogehoge
            !0 = !{}
            define dso_local i32 @main(i32 %0, i32 %1) #0 {
                ret void
            }
            attributes #0 = { noinline "abcde" = ;fff
            ;ff
                                            "ff" "xxx"}
        "#,
    );
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.source_filename, "c.c");
    assert_eq!(
        result.target.datalayout,
        "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
    );
    assert_eq!(result.target.triple, "x86_64-pc-linux-gnu");
    println!("{:?}", result);
}

#[test]
#[rustfmt::skip]
fn parse_module2() {
    use rustc_hash::FxHashMap;
    let result = parse(include_str!("../../../examples/ret42.ll"));
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.source_filename, "c.c");
    assert_eq!(
        result.target.datalayout,
        "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
    );
    assert_eq!(result.target.triple, "x86_64-pc-linux-gnu");
    let attrs = vec![(
        0u32,
        vec![
            Attribute::NoInline,
            Attribute::NoUnwind,
            Attribute::OptNone,
            Attribute::UWTable, 
            Attribute::StringAttribute {kind: "correctly-rounded-divide-sqrt-fp-math".to_string(), value: "false"                     .to_string()},
            Attribute::StringAttribute {kind: "disable-tail-calls"                   .to_string(), value: "false"                     .to_string()},
            Attribute::StringAttribute {kind: "frame-pointer"                        .to_string(), value: "all"                       .to_string()},
            Attribute::StringAttribute {kind: "less-precise-fpmad"                   .to_string(), value: "false"                     .to_string()},
            Attribute::StringAttribute {kind: "min-legal-vector-width"               .to_string(), value: "0"                         .to_string()},
            Attribute::StringAttribute {kind: "no-infs-fp-math"                      .to_string(), value: "false"                     .to_string()},
            Attribute::StringAttribute {kind: "no-jump-tables"                       .to_string(), value: "false"                     .to_string()},
            Attribute::StringAttribute {kind: "no-nans-fp-math"                      .to_string(), value: "false"                     .to_string()},
            Attribute::StringAttribute {kind: "no-signed-zeros-fp-math"              .to_string(), value: "false"                     .to_string()},
            Attribute::StringAttribute {kind: "no-trapping-math"                     .to_string(), value: "false"                     .to_string()},
            Attribute::StringAttribute {kind: "stack-protector-buffer-size"          .to_string(), value: "8"                         .to_string()},
            Attribute::StringAttribute {kind: "target-cpu"                           .to_string(), value: "x86-64"                    .to_string()},
            Attribute::StringAttribute {kind: "target-features"                      .to_string(), value: "+fxsr,+mmx,+sse,+sse2,+x87".to_string()},
            Attribute::StringAttribute {kind: "unsafe-fp-math"                       .to_string(), value: "false"                     .to_string()},
            Attribute::StringAttribute {kind: "use-soft-float"                       .to_string(), value: "false"                     .to_string()},
        ],
    )]
    .into_iter()
    .collect::<FxHashMap<u32, Vec<Attribute>>>();
    for (key1, val1) in &result.attributes {
        assert!(&attrs[key1] == val1)
    }
    println!("{:?}", result);
}

#[test]
fn parse_metadata1() {
    use super::super::value::ConstantInt;
    let m = Module::new();
    fn t(s: &str) -> String {
        s.to_string()
    }
    fn nm(s: &str) -> Name {
        Name::Name(s.to_string())
    }
    fn nn(s: usize) -> Name {
        Name::Number(s)
    }
    fn i32(s: i32) -> ConstantInt {
        ConstantInt::Int32(s)
    }

    let s = "
        !llvm.module.flags = !{!0}
        !llvm.ident = !{!1}
        !0 = !{i32 1, !\"wchar_size\", i32 4}
        !1 = !{!\"clang version 10.0.0-4ubuntu1 \"}
        !llvm.module.flags = !{!0, !1, !2}
        !0 = !{i32 7, !\"PIC Level\", i32 2}
        !1 = !{i32 7, !\"PIE Level\", i32 2}
        !2 = !{i32 2, !\"RtLibUseGOT\", i32 1}
        !3 = !{}
        !4 = !{i32 2849348}
        !4 = !{i32 2849319}
        !4 = !{i32 2849383}";
    use nom::multi::many1;
    assert_eq!(
        many1(parse_metadata(&m.types))(s),
        Ok((
            "",
            vec![
                (
                    nm("llvm.module.flags"),
                    Meta::Metas(vec![Meta::Name(nn(0))])
                ),
                (nm("llvm.ident"), Meta::Metas(vec![Meta::Name(nn(1))])),
                (
                    nn(0),
                    Meta::Metas(vec![
                        Meta::Int(i32(1)),
                        Meta::String(t("wchar_size")),
                        Meta::Int(i32(4))
                    ])
                ),
                (
                    nn(1),
                    Meta::Metas(vec![Meta::String(t("clang version 10.0.0-4ubuntu1 "))])
                ),
                (
                    nm("llvm.module.flags"),
                    Meta::Metas(vec![
                        Meta::Name(nn(0)),
                        Meta::Name(nn(1)),
                        Meta::Name(nn(2))
                    ])
                ),
                (
                    nn(0),
                    Meta::Metas(vec![
                        Meta::Int(i32(7)),
                        Meta::String(t("PIC Level")),
                        Meta::Int(i32(2))
                    ])
                ),
                (
                    nn(1),
                    Meta::Metas(vec![
                        Meta::Int(i32(7)),
                        Meta::String(t("PIE Level")),
                        Meta::Int(i32(2))
                    ])
                ),
                (
                    nn(2),
                    Meta::Metas(vec![
                        Meta::Int(i32(2)),
                        Meta::String(t("RtLibUseGOT")),
                        Meta::Int(i32(1))
                    ])
                ),
                (nn(3), Meta::Metas(vec![])),
                (nn(4), Meta::Metas(vec![Meta::Int(i32(2849348))])),
                (nn(4), Meta::Metas(vec![Meta::Int(i32(2849319))])),
                (nn(4), Meta::Metas(vec![Meta::Int(i32(2849383))])),
            ]
        ))
    );
}