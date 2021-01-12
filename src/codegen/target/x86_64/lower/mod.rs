use crate::codegen::{
    calling_conv::CallingConv,
    function::instruction::Instruction as MachInstruction,
    lower::{Lower as LowerTrait, LoweringContext},
    register::VReg,
    target::x86_64::{
        instruction::{InstructionData, MemoryOperand, Opcode, Operand as MOperand, OperandData},
        register::{RegClass, GR32},
        X86_64,
    },
};
use crate::ir::{
    function::{
        basic_block::BasicBlockId,
        instruction::{ICmpCond, Instruction as IrInstruction, InstructionId, Operand},
        Data as IrData,
    },
    types::{Type, TypeId},
    value::{ConstantData, ConstantInt, Value, ValueId},
};

#[derive(Clone, Copy)]
pub struct Lower {}

impl Lower {
    pub fn new() -> Self {
        Lower {}
    }
}

impl<CC: CallingConv<RegClass>> LowerTrait<X86_64<CC>> for Lower {
    fn lower(ctx: &mut LoweringContext<X86_64<CC>>, inst: &IrInstruction) {
        lower(ctx, inst)
    }
}

fn lower<CC: CallingConv<RegClass>>(ctx: &mut LoweringContext<X86_64<CC>>, inst: &IrInstruction) {
    match inst.operand {
        Operand::Alloca {
            ref tys,
            ref num_elements,
            align,
        } => lower_alloca(ctx, inst.id.unwrap(), tys, num_elements, align),
        Operand::Phi {
            ty,
            ref args,
            ref blocks,
        } => lower_phi(ctx, inst.id.unwrap(), ty, args, blocks),
        Operand::Load {
            ref tys,
            addr,
            align,
        } => lower_load(ctx, inst.id.unwrap(), tys, addr, align),
        Operand::Store {
            ref tys,
            ref args,
            align,
        } => lower_store(ctx, tys, args, align),
        Operand::IntBinary { ty, ref args, .. } => lower_add(ctx, inst.id.unwrap(), ty, args),
        Operand::Br { block } => lower_br(ctx, block),
        Operand::CondBr { arg, blocks } => lower_condbr(ctx, arg, blocks),
        Operand::Ret { val: None, .. } => todo!(),
        Operand::Ret { val: Some(val), ty } => lower_return(ctx, ty, val),
        _ => todo!(),
    }
}

fn lower_alloca<CC: CallingConv<RegClass>>(
    ctx: &mut LoweringContext<X86_64<CC>>,
    id: InstructionId,
    tys: &[TypeId],
    _num_elements: &ConstantData,
    _align: u32,
) {
    let slot_id = ctx.slots.add_slot(tys[0]);
    ctx.inst_id_to_slot_id.insert(id, slot_id);
}

fn lower_phi<CC: CallingConv<RegClass>>(
    ctx: &mut LoweringContext<X86_64<CC>>,
    id: InstructionId,
    ty: TypeId,
    args: &[ValueId],
    blocks: &[BasicBlockId],
) {
    let output = new_empty_inst_output(ctx, ty, id);
    let mut operands = vec![MOperand::output(OperandData::VReg(output))];
    for (arg, block) in args.iter().zip(blocks.iter()) {
        operands.push(match ctx.ir_data.value_ref(*arg) {
            Value::Instruction(id) => {
                MOperand::input(OperandData::VReg(get_or_generate_inst_output(ctx, *id)))
            }
            Value::Constant(ConstantData::Int(ConstantInt::Int32(i))) => {
                MOperand::new(OperandData::Int32(*i))
            }
            _ => todo!(),
        });
        operands.push(MOperand::new(OperandData::Block(ctx.block_map[block])))
    }
    ctx.inst_seq.push(MachInstruction::new(InstructionData {
        opcode: Opcode::Phi,
        operands,
    }));
}

fn lower_load<CC: CallingConv<RegClass>>(
    ctx: &mut LoweringContext<X86_64<CC>>,
    id: InstructionId,
    tys: &[TypeId],
    addr: ValueId,
    _align: u32,
) {
    let mut slot = None;

    match ctx.ir_data.value_ref(addr) {
        Value::Instruction(id) => {
            if let Some(slot_id) = ctx.inst_id_to_slot_id.get(id) {
                slot = Some(*slot_id);
            }
        }
        _ => todo!(),
    }

    if let Some(slot) = slot {
        if matches!(&*ctx.types.get(tys[0]), Type::Int(32)) {
            let vreg = new_empty_inst_output(ctx, tys[0], id);
            ctx.inst_seq.push(MachInstruction::new(InstructionData {
                opcode: Opcode::MOVrm32,
                operands: vec![
                    MOperand::output(OperandData::VReg(vreg)),
                    MOperand::input(OperandData::Mem(MemoryOperand::Slot(slot))),
                ],
            }));
            return;
        }
    }

    todo!()
}

fn lower_store<CC: CallingConv<RegClass>>(
    ctx: &mut LoweringContext<X86_64<CC>>,
    _tys: &[TypeId],
    args: &[ValueId],
    _align: u32,
) {
    let mut slot = None;

    match ctx.ir_data.value_ref(args[1]) {
        Value::Instruction(id) => {
            if let Some(slot_id) = ctx.inst_id_to_slot_id.get(id) {
                slot = Some(*slot_id);
            }
        }
        _ => todo!(),
    }

    let mut const_int = None;
    let mut inst = None;

    match ctx.ir_data.value_ref(args[0]) {
        Value::Constant(ConstantData::Int(int)) => const_int = Some(*int),
        Value::Instruction(id) => inst = Some(*id),
        _ => {}
    }

    match (slot, inst) {
        (Some(slot), Some(id)) => {
            let inst = get_or_generate_inst_output(ctx, id);
            ctx.inst_seq
                .append(&mut vec![MachInstruction::new(InstructionData {
                    opcode: Opcode::MOVmi32,
                    operands: vec![
                        MOperand::output(OperandData::Mem(MemoryOperand::Slot(slot))),
                        MOperand::input(OperandData::VReg(inst)),
                    ],
                })]);
            return;
        }
        _ => {}
    }

    match (slot, const_int) {
        (Some(slot), Some(ConstantInt::Int32(imm))) => {
            ctx.inst_seq.append(&mut vec![MachInstruction {
                id: None,
                data: InstructionData {
                    opcode: Opcode::MOVmi32,
                    operands: vec![
                        MOperand::output(OperandData::Mem(MemoryOperand::Slot(slot))),
                        MOperand::input(OperandData::Int32(imm)),
                    ],
                },
            }]);
            return;
        }
        _ => todo!(),
    }
}

fn lower_add<CC: CallingConv<RegClass>>(
    ctx: &mut LoweringContext<X86_64<CC>>,
    id: InstructionId,
    ty: TypeId,
    args: &[ValueId],
) {
    let lhs;
    let rhs = ctx.ir_data.value_ref(args[1]);
    let output;

    if let Value::Instruction(l) = ctx.ir_data.value_ref(args[0]) {
        lhs = get_or_generate_inst_output(ctx, *l);
        output = new_empty_inst_output(ctx, ty, id);
    } else {
        panic!();
    };

    let insert_move = |ctx: &mut LoweringContext<X86_64<CC>>| {
        ctx.inst_seq.push(MachInstruction {
            id: None,
            data: InstructionData {
                opcode: Opcode::MOVrr32,
                operands: vec![
                    MOperand::output(OperandData::VReg(output)),
                    MOperand::input(OperandData::VReg(lhs)),
                ],
            },
        })
    };

    if let Value::Instruction(rhs) = rhs {
        let rhs = get_or_generate_inst_output(ctx, *rhs);
        insert_move(ctx);
        ctx.inst_seq.push(MachInstruction {
            id: None,
            data: InstructionData {
                opcode: Opcode::ADDrr32,
                operands: vec![
                    MOperand::input_output(OperandData::VReg(output)),
                    MOperand::input(OperandData::VReg(rhs)),
                ],
            },
        });
        return;
    }

    if let Value::Constant(ConstantData::Int(ConstantInt::Int32(rhs))) = rhs {
        insert_move(ctx);
        ctx.inst_seq.push(MachInstruction {
            id: None,
            data: InstructionData {
                opcode: Opcode::ADDrr32,
                operands: vec![
                    MOperand::input_output(OperandData::VReg(output)),
                    MOperand::new(OperandData::Int32(*rhs)),
                ],
            },
        });
        return;
    }

    todo!()
}

fn lower_br<CC: CallingConv<RegClass>>(ctx: &mut LoweringContext<X86_64<CC>>, block: BasicBlockId) {
    ctx.inst_seq.push(MachInstruction {
        id: None,
        data: InstructionData {
            opcode: Opcode::JMP,
            operands: vec![MOperand::new(OperandData::Block(ctx.block_map[&block]))],
        },
    })
}

fn lower_condbr<CC: CallingConv<RegClass>>(
    ctx: &mut LoweringContext<X86_64<CC>>,
    arg: ValueId,
    blocks: [BasicBlockId; 2],
) {
    fn is_icmp<'a>(
        data: &'a IrData,
        val: &Value,
    ) -> Option<(&'a TypeId, &'a [ValueId; 2], &'a ICmpCond)> {
        match val {
            Value::Instruction(id) => {
                let inst = data.inst_ref(*id);
                match &inst.operand {
                    Operand::ICmp { ty, args, cond } => return Some((ty, args, cond)),
                    _ => return None,
                }
            }
            _ => return None,
        }
    }

    let arg = ctx.ir_data.value_ref(arg);

    if let Some((_ty, args, cond)) = is_icmp(ctx.ir_data, arg) {
        let lhs = ctx.ir_data.value_ref(args[0]);
        let rhs = ctx.ir_data.value_ref(args[1]);
        match (lhs, rhs) {
            (
                Value::Instruction(lhs),
                Value::Constant(ConstantData::Int(ConstantInt::Int32(rhs))),
            ) => {
                let lhs = get_or_generate_inst_output(ctx, *lhs);
                ctx.inst_seq.push(MachInstruction::new(InstructionData {
                    opcode: Opcode::CMPri32,
                    operands: vec![
                        MOperand::input(OperandData::VReg(lhs)),
                        MOperand::new(OperandData::Int32(*rhs)),
                    ],
                }));
            }
            _ => todo!(),
        }

        ctx.inst_seq.push(MachInstruction::new(InstructionData {
            opcode: match cond {
                ICmpCond::Eq => Opcode::JE,
                ICmpCond::Ne => Opcode::JNE,
                ICmpCond::Sle => Opcode::JLE,
                ICmpCond::Slt => Opcode::JL,
                ICmpCond::Sge => Opcode::JGE,
                ICmpCond::Sgt => Opcode::JG,
                _ => todo!(),
            },
            operands: vec![MOperand::new(OperandData::Block(ctx.block_map[&blocks[0]]))],
        }));
        ctx.inst_seq.push(MachInstruction::new(InstructionData {
            opcode: Opcode::JMP,
            operands: vec![MOperand::new(OperandData::Block(ctx.block_map[&blocks[1]]))],
        }));
        return;
    }

    todo!()
}

fn lower_return<CC: CallingConv<RegClass>>(
    ctx: &mut LoweringContext<X86_64<CC>>,
    _ty: TypeId,
    value: ValueId,
) {
    let value = ctx.ir_data.value_ref(value);
    match value {
        Value::Constant(ConstantData::Int(ConstantInt::Int32(i))) => {
            ctx.inst_seq.push(MachInstruction {
                id: None,
                data: InstructionData {
                    opcode: Opcode::MOVri32,
                    operands: vec![
                        MOperand::output(OperandData::Reg(GR32::EAX.into())),
                        MOperand::input(OperandData::Int32(*i)),
                    ],
                },
            });
        }
        Value::Instruction(id) => {
            let vreg = get_or_generate_inst_output(ctx, *id);
            ctx.inst_seq.push(MachInstruction {
                id: None,
                data: InstructionData {
                    opcode: Opcode::MOVrr32,
                    operands: vec![
                        MOperand::output(OperandData::Reg(GR32::EAX.into())),
                        MOperand::input(OperandData::VReg(vreg)),
                    ],
                },
            });
        }
        _ => todo!(),
    }
    ctx.inst_seq.push(MachInstruction {
        id: None,
        data: InstructionData {
            opcode: Opcode::RET,
            operands: vec![],
        },
    });
}

fn get_or_generate_inst_output<CC: CallingConv<RegClass>>(
    ctx: &mut LoweringContext<X86_64<CC>>,
    id: InstructionId,
) -> VReg {
    if let Some(vreg) = ctx.inst_id_to_vreg.get(&id) {
        return *vreg;
    }

    if ctx.ir_data.inst_ref(id).parent != ctx.cur_block {
        // The instruction indexed as `id` must be placed in another basic block
        let v = ctx.vregs.add_vreg_data(ctx.types.base().void());
        ctx.inst_id_to_vreg.insert(id, v);
        return v;
    }

    // What about instruction scheduling?
    lower(ctx, ctx.ir_data.inst_ref(id));
    get_or_generate_inst_output(ctx, id)
}

fn new_empty_inst_output<CC: CallingConv<RegClass>>(
    ctx: &mut LoweringContext<X86_64<CC>>,
    ty: TypeId,
    id: InstructionId,
) -> VReg {
    if let Some(vreg) = ctx.inst_id_to_vreg.get(&id) {
        ctx.vregs.change_ty(*vreg, ty);
        return *vreg;
    }
    let vreg = ctx.vregs.add_vreg_data(ty);
    ctx.inst_id_to_vreg.insert(id, vreg);
    vreg
}
