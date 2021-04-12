pub mod function;
pub mod module;

use crate::{
    ir::{
        function::{
            basic_block::BasicBlockId, data::Data as IrData, instruction::Instruction as IrInst,
            Function as IrFunction,
        },
        module::Module as IrModule,
    },
    lower::{
        dag::function::{
            data::Data as DagData,
            node::{NodeData, NodeId},
        },
        isa::TargetIsa,
    },
};
use anyhow::Result;
use id_arena::Arena;
use rustc_hash::FxHashMap;
use std::{error::Error as StdError, fmt};

pub trait IrToDag<T: TargetIsa> {
    type NodeData: NodeData;
    fn convert(ctx: &mut Context<T>, inst: &IrInst) -> Result<NodeId<Self::NodeData>>;
}

pub struct Context<'a, T: TargetIsa> {
    pub isa: &'a T,
    pub ir_data: &'a IrData,
    pub dag_data: &'a mut DagData<<T::IrToDag as IrToDag<T>>::NodeData>,
    pub block: BasicBlockId,
}

#[derive(Debug)]
pub enum Error {
    Todo(&'static str),
}

pub fn convert_module<'a, T: TargetIsa>(
    isa: &'a T,
    module: &'a IrModule,
) -> Result<module::Module<'a, T>> {
    let mut functions: Arena<function::Function<'a, T>> = Arena::new();

    for (_, ir_func) in &module.functions {
        let dag_func = convert_function(isa, ir_func)?;
        functions.alloc(dag_func);
    }

    Ok(module::Module::new(isa, module))
}

pub fn convert_function<'a, T: TargetIsa>(
    isa: &'a T,
    function: &'a IrFunction,
) -> Result<function::Function<'a, T>> {
    let mut dag_data = DagData::new();
    let mut layout = function::layout::Layout::new();
    let mut block_map = FxHashMap::default();

    // Create dag basic blocks
    for block_id in function.layout.block_iter() {
        let new_block_id = dag_data.create_block();
        layout.append_block(new_block_id);
        block_map.insert(block_id, new_block_id);
    }

    // Insert preds and succs
    for block_id in function.layout.block_iter() {
        let new_block_id = block_map[&block_id];
        let block = &function.data.basic_blocks[block_id];
        let new_block = dag_data.basic_blocks.get_mut(new_block_id).unwrap();
        for pred in &block.preds {
            new_block.preds.insert(block_map[pred]);
        }
        for succ in &block.succs {
            new_block.succs.insert(block_map[succ]);
        }
    }

    // let mut dag_data = DagData::new();

    for (_i, block_id) in function.layout.block_iter().enumerate() {
        let mut ctx = Context {
            isa,
            ir_data: &function.data,
            dag_data: &mut dag_data,
            block: block_id,
        };
        for inst_id in function.layout.inst_iter(block_id).rev() {
            let inst = function.data.inst_ref(inst_id);
            T::IrToDag::convert(&mut ctx, inst)?;
        }
    }

    Ok(function::Function::new(isa, function))
}

impl StdError for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Todo(msg) => write!(f, "Todo({})", msg),
        }
    }
}
