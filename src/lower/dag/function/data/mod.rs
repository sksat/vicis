use crate::ir::function::basic_block::{BasicBlock, BasicBlockId};
use crate::lower::dag::function::instruction::{Instruction, InstructionData, InstructionId};
use id_arena::Arena;
// use rustc_hash::FxHashMap;

pub struct Data<InstData: InstructionData> {
    pub instructions: Arena<Instruction<InstData>>,
    pub basic_blocks: Arena<BasicBlock>,
}

impl<InstData: InstructionData> Data<InstData> {
    pub fn new() -> Self {
        Self {
            instructions: Arena::new(),
            basic_blocks: Arena::new(),
        }
    }

    pub fn create_block(&mut self) -> BasicBlockId {
        self.basic_blocks.alloc(BasicBlock::new())
    }

    pub fn create_inst(&mut self, mut inst: Instruction<InstData>) -> InstructionId<InstData> {
        self.instructions.alloc_with_id(|id| {
            inst.id = Some(id);
            inst
        })
    }

    pub fn block_ref(&self, id: BasicBlockId) -> &BasicBlock {
        &self.basic_blocks[id]
    }

    // TODO: Is this the right way?
    pub fn block_ref_mut(&mut self, id: BasicBlockId) -> &mut BasicBlock {
        &mut self.basic_blocks[id]
    }

    pub fn inst_ref(&self, id: InstructionId<InstData>) -> &Instruction<InstData> {
        &self.instructions[id]
    }

    pub fn inst_ref_mut(&mut self, id: InstructionId<InstData>) -> &mut Instruction<InstData> {
        &mut self.instructions[id]
    }
}
