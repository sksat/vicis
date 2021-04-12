pub mod function;

use crate::{
    ir::{
        function::instruction::{Instruction as IrInst, Operand},
        types::TypeId,
        value::{ConstantData, ConstantInt, Value, ValueId},
    },
    lower::dag::{
        function::node::{Node, NodeId},
        Context, Error, IrToDag as ITD,
    },
    targets::x86_64::{
        dag::function::node::{Inst, NodeData, Opcode},
        X86_64,
    },
};
use anyhow::Result;

pub struct IrToDag {}

impl ITD<X86_64> for IrToDag {
    type NodeData = NodeData;
    fn convert(ctx: &mut Context<X86_64>, inst: &IrInst) -> Result<NodeId<NodeData>> {
        convert(ctx, inst)
    }
}

fn convert(ctx: &mut Context<X86_64>, inst: &IrInst) -> Result<NodeId<NodeData>> {
    match inst.operand {
        Operand::Ret { val: None, .. } => Err(Error::Todo("ret void").into()),
        Operand::Ret { ty, val: Some(val) } => convert_ret(ctx, ty, val),
        _ => Err(Error::Todo("unimplemented").into()),
    }
}

fn convert_ret(ctx: &mut Context<X86_64>, _ty: TypeId, val: ValueId) -> Result<NodeId<NodeData>> {
    let val = convert_value(ctx, ctx.ir_data.value_ref(val))?;
    let ret = ctx.dag_data.create_node(Node::new(
        NodeData::Inst(Inst::new(Opcode::Ret, vec![val])),
        ctx.block,
    ));
    Ok(ret)
}

fn convert_value(ctx: &mut Context<X86_64>, val: &Value) -> Result<NodeId<NodeData>> {
    match val {
        // Value::Instruction(id) => {}
        // Value::Argument(arg) => {}
        Value::Constant(ConstantData::Int(ConstantInt::Int32(i))) => Ok(ctx
            .dag_data
            .create_node(Node::new(NodeData::Int32(*i), ctx.block))),
        _ => Err(Error::Todo("convert value").into()),
    }
}
