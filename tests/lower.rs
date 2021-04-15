use std::fs::read_to_string;
use vicis::{ir::module, lower::dag::convert_module, targets::x86_64::X86_64};

#[test]
fn ret42() {
    let asm = read_to_string("./tests/codegen/ret42.ll").unwrap();
    let module = module::parse_assembly(asm.as_str()).unwrap();
    println!("{:?}", module);
    convert_module(&X86_64, &module).unwrap();
}
