use crate::{ir::module::Module as IrModule, lower::isa::TargetIsa};

pub struct Module<'a, T: TargetIsa> {
    pub isa: &'a T,
    pub base: &'a IrModule,
}

impl<'a, T: TargetIsa> Module<'a, T> {
    pub fn new(isa: &'a T, base: &'a IrModule) -> Self {
        Self { isa, base }
    }
}
