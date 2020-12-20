pub mod parser;

pub use parser::parse;

use super::{
    basic_block::{BasicBlock, BasicBlockId},
    instruction::{Instruction, InstructionId},
    module::{name::Name, preemption_specifier::PreemptionSpecifier},
    types::{TypeId, Types},
};
use id_arena::Arena;
use rustc_hash::FxHashMap;
use std::fmt;

pub struct Function {
    pub name: String,
    pub is_var_arg: bool,
    pub result_ty: TypeId,
    pub params: Vec<Parameter>,
    pub preemption_specifier: PreemptionSpecifier,
    // pub attributes:
    pub data: Data,
    pub layout: Layout,
    pub types: Types,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: Name,
    pub ty: TypeId,
    // pub attributes:
}

pub struct Data {
    pub instructions: Arena<Instruction>,
    pub basic_blocks: Arena<BasicBlock>,
}

pub struct Layout {
    basic_blocks: FxHashMap<BasicBlockId, BasicBlockNode>,
    instructions: FxHashMap<InstructionId, InstructionNode>,
    first_block: Option<BasicBlockId>,
    last_block: Option<BasicBlockId>,
}

pub struct BasicBlockNode {
    prev: Option<BasicBlockId>,
    next: Option<BasicBlockId>,
    first_inst: Option<InstructionId>,
    last_inst: Option<InstructionId>,
}

pub struct InstructionNode {
    prev: Option<InstructionId>,
    next: Option<InstructionId>,
}

pub struct BasicBlockIter<'a> {
    layout: &'a Layout,
    cur: Option<BasicBlockId>,
}

impl Data {
    pub fn new() -> Self {
        Self {
            instructions: Arena::new(),
            basic_blocks: Arena::new(),
        }
    }

    pub fn create_block(&mut self) -> BasicBlockId {
        self.basic_blocks.alloc(BasicBlock::new())
    }

    pub fn create_inst(&mut self, inst: Instruction) -> InstructionId {
        self.instructions.alloc(inst)
    }
}

impl Layout {
    pub fn new() -> Self {
        Self {
            basic_blocks: FxHashMap::default(),
            instructions: FxHashMap::default(),
            first_block: None,
            last_block: None,
        }
    }

    pub fn block_iter<'a>(&'a self) -> BasicBlockIter<'a> {
        BasicBlockIter {
            layout: self,
            cur: self.first_block,
        }
    }

    pub fn append_block(&mut self, block: BasicBlockId) {
        self.basic_blocks.entry(block).or_insert(BasicBlockNode {
            prev: self.last_block,
            next: None,
            first_inst: None,
            last_inst: None,
        });

        if let Some(last_block) = self.last_block {
            self.basic_blocks.get_mut(&last_block).unwrap().next = Some(block);
            self.basic_blocks.get_mut(&block).unwrap().prev = Some(last_block);
        }

        self.last_block = Some(block);

        if self.first_block.is_none() {
            self.first_block = Some(block)
        }
    }

    pub fn append_inst(&mut self, inst: InstructionId, block: BasicBlockId) {
        self.instructions.entry(inst).or_insert(InstructionNode {
            prev: self.basic_blocks[&block].last_inst,
            next: None,
        });

        if let Some(last_inst) = self.basic_blocks[&block].last_inst {
            self.instructions.get_mut(&last_inst).unwrap().next = Some(inst);
            self.instructions.get_mut(&inst).unwrap().prev = Some(last_inst);
        }

        self.basic_blocks.get_mut(&block).unwrap().last_inst = Some(inst);

        if self.basic_blocks[&block].first_inst.is_none() {
            self.basic_blocks.get_mut(&block).unwrap().first_inst = Some(inst);
        }
    }
}

impl<'a> Iterator for BasicBlockIter<'a> {
    type Item = BasicBlockId;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.cur?;
        self.cur = self.layout.basic_blocks[&cur].next;
        Some(cur)
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "define ")?;
        write!(f, "{:?} ", self.preemption_specifier)?;
        write!(f, "{} ", self.types.to_string(self.result_ty))?;
        write!(f, "@{}(", self.name)?;
        for (i, param) in self.params.iter().enumerate() {
            write!(
                f,
                "{}{}",
                param.to_string(&self.types),
                if i == self.params.len() - 1 { "" } else { ", " }
            )?;
        }
        write!(f, ") ")?;
        write!(f, "{{\n")?;

        for block_id in self.layout.block_iter() {
            writeln!(f, "{:?}", block_id)?;
        }

        write!(f, "}}\n")?;
        Ok(())
    }
}

impl Parameter {
    pub fn to_string(&self, types: &Types) -> String {
        format!("{} %{:?}", types.to_string(self.ty), self.name)
    }
}
