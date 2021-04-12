use crate::ir::function::basic_block::BasicBlockId;
use id_arena::Id;
use std::fmt;

pub type InstructionId<Data> = Id<Instruction<Data>>;

pub trait InstructionData: Clone + fmt::Debug {}

pub struct Instruction<Data: InstructionData> {
    pub id: Option<InstructionId<Data>>,
    pub data: Data,
    pub parent: BasicBlockId,
}
