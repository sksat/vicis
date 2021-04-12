use crate::lower::dag::function::node::{NodeData as ND, NodeId};

#[derive(Debug, Clone)]
pub enum NodeData {
    Inst(Inst<Self>),
    Int32(i32),
}

#[derive(Debug, Clone)]
pub struct Inst<Data: ND> {
    pub opcode: Opcode,
    pub args: Vec<NodeId<Data>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Opcode {
    Alloca,
    Phi,
    Load,
    Store,
    Add,
    Sub,
    Mul,
    ICmp,
    Sext,
    Zext,
    GetElementPtr,
    Call,
    Br,
    CondBr,
    Ret,
    Invalid,
}

impl<Data: ND> Inst<Data> {
    pub fn new(opcode: Opcode, args: Vec<NodeId<Data>>) -> Self {
        Self { opcode, args }
    }
}

impl ND for NodeData {}
