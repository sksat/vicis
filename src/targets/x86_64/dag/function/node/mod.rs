use crate::lower::dag::function::node::{NodeData as ND, NodeId};

#[derive(Debug, Clone)]
pub enum NodeData {
    Root,
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

impl ND for NodeData {
    fn root() -> Self {
        Self::Root
    }

    fn args(&self) -> &[NodeId<Self>] {
        match self {
            Self::Root => &[],
            Self::Inst(inst) => &inst.args,
            Self::Int32(_) => &[],
        }
    }

    fn dot_label(&self) -> String {
        match self {
            Self::Root => "Root".to_string(),
            Self::Inst(inst) => format!("{:?}", inst.opcode),
            Self::Int32(i) => format!("{}", i),
        }
    }
}
