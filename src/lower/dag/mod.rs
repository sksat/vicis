pub mod function;
pub mod module;

use crate::ir::{function::Function as IrFunction, module::Module as IrModule};
use id_arena::Arena;

pub fn convert_module<'a>(module: &'a IrModule) -> module::Module<'a> {
    let mut functions: Arena<function::Function<'a>> = Arena::new();
    for (_, ir_func) in &module.functions {
        let dag_func = convert_function(ir_func);
        functions.alloc(dag_func);
    }
    module::Module::new(module)
}

pub fn convert_function<'a>(function: &'a IrFunction) -> function::Function<'a> {
    function::Function::new(function)
}
