use crate::{
    ir::function::instruction::{Instruction as IrInst, Operand},
    lower::dag::{Context, Error, IrToDag as ITD},
    targets::x86_64::X86_64,
};
use anyhow::Result;

pub struct IrToDag {}

impl ITD<X86_64> for IrToDag {
    fn convert(ctx: &mut Context<X86_64>, inst: &IrInst) -> Result<()> {
        convert(ctx, inst)
    }
}

fn convert(_ctx: &mut Context<X86_64>, inst: &IrInst) -> Result<()> {
    match inst.operand {
        Operand::Ret { val: None, .. } => Err(Error::Todo("todo").into()),
        _ => Err(Error::Todo("unimplemented").into()),
    }
}
