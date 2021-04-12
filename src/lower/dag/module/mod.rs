use crate::ir::module::Module as IrModule;

pub struct Module<'a> {
    pub base: &'a IrModule,
}

impl<'a> Module<'a> {
    pub fn new(base: &'a IrModule) -> Self {
        Self { base }
    }
}
