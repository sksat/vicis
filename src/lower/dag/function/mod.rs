pub mod data;
pub mod layout;
pub mod node;

use crate::{ir::function::Function as IrFunction, lower::isa::TargetIsa};

pub struct Function<'a, T: TargetIsa> {
    pub isa: &'a T,
    pub base: &'a IrFunction,
}

impl<'a, T: TargetIsa> Function<'a, T> {
    pub fn new(isa: &'a T, base: &'a IrFunction) -> Self {
        Self { isa, base }
    }
}
