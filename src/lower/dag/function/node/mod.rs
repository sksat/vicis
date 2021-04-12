use crate::ir::function::basic_block::BasicBlockId;
use id_arena::Id;
use std::fmt;

pub type NodeId<Data> = Id<Node<Data>>;

pub trait NodeData: Clone + fmt::Debug {
    fn root() -> Self;
    fn dot_label(&self) -> String;
    fn args(&self) -> &[NodeId<Self>];
}

#[derive(Debug)]
pub struct Node<Data: NodeData> {
    pub id: Option<NodeId<Data>>,
    pub data: Data,
    pub parent: BasicBlockId,
    pub chain: Option<NodeId<Data>>,
}

impl<Data: NodeData> Node<Data> {
    pub fn new(data: Data, parent: BasicBlockId) -> Self {
        Self {
            id: None,
            data,
            parent,
            chain: None,
        }
    }

    pub fn root(parent: BasicBlockId) -> Self {
        Self {
            id: None,
            data: Data::root(),
            parent,
            chain: None,
        }
    }

    pub fn set_chain(&mut self, id: NodeId<Data>) {
        self.chain = Some(id)
    }
}
