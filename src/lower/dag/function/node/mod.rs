use crate::ir::function::basic_block::BasicBlockId;
use id_arena::Id;
use std::fmt;

pub type NodeId<Data> = Id<Node<Data>>;

pub trait NodeData: Clone + fmt::Debug {}

pub struct Node<Data: NodeData> {
    pub id: Option<NodeId<Data>>,
    pub data: Data,
    pub parent: BasicBlockId,
}

impl<Data: NodeData> Node<Data> {
    pub fn new(data: Data, parent: BasicBlockId) -> Self {
        Self {
            id: None,
            data,
            parent,
        }
    }
}
