pub mod data;

use crate::ir::function::Function as IrFunction;

pub struct Function<'a> {
    pub base: &'a IrFunction,
}

impl<'a> Function<'a> {
    pub fn new(base: &'a IrFunction) -> Self {
        Self { base }
    }
}
