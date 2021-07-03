use vicis_ir::{ir::module, pass::transform::mem2reg::Mem2Reg};

#[test]
fn mem2reg_1() {
    let ir = r#"
define dso_local i32 @main() {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  store i32 0, i32* %1, align 4
  store i32 234, i32* %2, align 4
  %3 = load i32, i32* %2, align 4
  %4 = add nsw i32 %3, 123
  ret i32 %4
}"#;
    let mut module = module::parse_assembly(ir).expect("failed to parse ir");
    for (_, func) in module.functions_mut() {
        Mem2Reg::new(func).run();
        println!("{:?}", func);
    }
}

#[test]
fn mem2reg_2() {
    let ir = r#"
define dso_local i32 @main() {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store i32 0, i32* %1, align 4
  store i32 1, i32* %2, align 4
  %6 = load i32, i32* %2, align 4
  %7 = add nsw i32 %6, 1
  store i32 %7, i32* %3, align 4
  %8 = load i32, i32* %2, align 4
  %9 = load i32, i32* %3, align 4
  %10 = sub nsw i32 %8, %9
  store i32 %10, i32* %4, align 4
  store i32 3, i32* %2, align 4
  %11 = load i32, i32* %2, align 4
  store i32 %11, i32* %5, align 4
  %12 = load i32, i32* %3, align 4
  %13 = load i32, i32* %5, align 4
  %14 = add nsw i32 %12, %13
  ret i32 %14
}"#;
    let mut module = module::parse_assembly(ir).expect("failed to parse ir");
    for (_, func) in module.functions_mut() {
        Mem2Reg::new(func).run();
        println!("{:?}", func);
    }
}
