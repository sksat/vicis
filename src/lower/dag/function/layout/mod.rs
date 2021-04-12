use crate::{
    ir::function::basic_block::BasicBlockId,
    lower::dag::function::node::{NodeData, NodeId},
};
use rustc_hash::FxHashMap;

pub struct Layout<ND: NodeData> {
    basic_blocks: FxHashMap<BasicBlockId, BasicBlockNode<ND>>,
    pub first_block: Option<BasicBlockId>,
    pub last_block: Option<BasicBlockId>,
}

pub struct BasicBlockNode<ND: NodeData> {
    _prev: Option<BasicBlockId>,
    next: Option<BasicBlockId>,
    node_root: Option<NodeId<ND>>,
}

// pub struct InstructionNode<ND: ND> {
//     block: Option<BasicBlockId>,
//     prev: Option<NodeId<ND>>,
//     next: Option<NodeId<ND>>,
// }

pub struct BasicBlockIter<'a, ND: NodeData> {
    layout: &'a Layout<ND>,
    cur: Option<BasicBlockId>,
}

// pub struct InstructionIter<'a, ND: ND> {
//     layout: &'a Layout<ND>,
//     block: BasicBlockId,
//     cur: Option<NodeId<ND>>,
// }

impl<ND: NodeData> Layout<ND> {
    pub fn new() -> Self {
        Self {
            basic_blocks: FxHashMap::default(),
            first_block: None,
            last_block: None,
        }
    }

    pub fn block_iter<'a>(&'a self) -> BasicBlockIter<'a, ND> {
        BasicBlockIter {
            layout: self,
            cur: self.first_block,
        }
    }

    pub fn next_block_of(&self, block: BasicBlockId) -> Option<BasicBlockId> {
        self.basic_blocks[&block].next
    }

    pub fn append_block(&mut self, block: BasicBlockId) {
        self.basic_blocks.entry(block).or_insert(BasicBlockNode {
            _prev: self.last_block,
            next: None,
            node_root: None,
        });

        if let Some(last_block) = self.last_block {
            self.basic_blocks.get_mut(&last_block).unwrap().next = Some(block);
            self.basic_blocks.get_mut(&block).unwrap()._prev = Some(last_block);
        }

        self.last_block = Some(block);

        if self.first_block.is_none() {
            self.first_block = Some(block)
        }
    }

    pub fn set_node_root(&mut self, root: NodeId<ND>, block: BasicBlockId) {
        self.basic_blocks.get_mut(&block).unwrap().node_root = Some(root);
    }

    pub fn node_root_of(&self, block: BasicBlockId) -> Option<NodeId<ND>> {
        self.basic_blocks[&block].node_root
    }
}

impl<'a, ND: NodeData> Iterator for BasicBlockIter<'a, ND> {
    type Item = BasicBlockId;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.cur?;
        self.cur = self.layout.basic_blocks[&cur].next;
        Some(cur)
    }
}
