use crate::ir::function::basic_block::{BasicBlock, BasicBlockId};
use crate::lower::dag::function::node::{Node, NodeData, NodeId};
use id_arena::Arena;

pub struct Data<D: NodeData> {
    pub nodes: Arena<Node<D>>,
    pub basic_blocks: Arena<BasicBlock>,
}

impl<D: NodeData> Data<D> {
    pub fn new() -> Self {
        Self {
            nodes: Arena::new(),
            basic_blocks: Arena::new(),
        }
    }

    pub fn create_block(&mut self) -> BasicBlockId {
        self.basic_blocks.alloc(BasicBlock::new())
    }

    pub fn create_node(&mut self, mut node: Node<D>) -> NodeId<D> {
        self.nodes.alloc_with_id(|id| {
            node.id = Some(id);
            node
        })
    }

    pub fn block_ref(&self, id: BasicBlockId) -> &BasicBlock {
        &self.basic_blocks[id]
    }

    // TODO: Is this the right way?
    pub fn block_ref_mut(&mut self, id: BasicBlockId) -> &mut BasicBlock {
        &mut self.basic_blocks[id]
    }

    pub fn node_ref(&self, id: NodeId<D>) -> &Node<D> {
        &self.nodes[id]
    }

    pub fn node_ref_mut(&mut self, id: NodeId<D>) -> &mut Node<D> {
        &mut self.nodes[id]
    }
}
