// use anyhow::Result;
use crate::lower::dag::IrToDag;

pub trait TargetIsa: Copy {
    // type InstInfo: InstructionInfo;
    // type RegClass: RegisterClass;
    // type RegInfo: RegisterInfo;
    // type Lower: lower::Lower<Self>;
    type IrToDag: IrToDag<Self>;

    // fn module_pass_list() -> Vec<fn(&mut Module<Self>) -> Result<()>>;
    // fn default_call_conv() -> CallConvKind;
    // fn type_size(types: &Types, ty: TypeId) -> u32;
}
