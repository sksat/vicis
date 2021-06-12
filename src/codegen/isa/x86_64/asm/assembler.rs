extern crate iced_x86;
extern crate memmap;

use iced_x86::{
    BlockEncoder, BlockEncoderOptions, Code, Instruction as IcedInst, InstructionBlock, Register,
};
use id_arena::Id;
use memmap::MmapMut;
use rustc_hash::FxHashMap;

use crate::codegen::{
    function::Function,
    isa::x86_64::{instruction::Opcode, X86_64},
    module::Module,
    register::Reg,
};

const BITNESS: u32 = 64;

type FuncId = Id<Function<X86_64>>;

pub struct Context {
    label_id: u64,
    instructions: Vec<IcedInst>,
    funcs_label: FxHashMap<FuncId, u64>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            label_id: 1,
            instructions: vec![],
            funcs_label: FxHashMap::default(),
        }
    }
}

impl Context {
    pub fn new_label(&mut self) -> u64 {
        let id = self.label_id;
        self.label_id += 1;
        id
    }

    pub fn add_func_label(&mut self, func: FuncId, label: u64) {
        self.funcs_label.insert(func, label);
    }

    pub fn add_instruction(&mut self, inst: IcedInst) {
        self.instructions.push(inst);
    }

    pub fn run_function(&self, func_id: FuncId) -> Option<()> {
        let label = self.funcs_label.get(&func_id)?;
        None
    }
}

pub fn assemble(ctx: &mut Context, module: &Module<X86_64>) {
    for (func_id, func) in &module.functions {
        assemble_function(ctx, func_id, func)
    }

    // Run

    let target_rip = 0x0000_0000_0000_0000;
    let block = InstructionBlock::new(&ctx.instructions, target_rip);
    let result = match BlockEncoder::encode(BITNESS, block, BlockEncoderOptions::RETURN_RELOC_INFOS)
    {
        Err(error) => panic!("Failed to encode it: {}", error),
        Ok(result) => result,
    };
    println!("const {:?}", result.reloc_infos);

    unsafe {
        let mut mem = MmapMut::map_anon(result.code_buffer.len()).unwrap();
        ::std::ptr::copy_nonoverlapping(
            result.code_buffer.as_ptr(),
            mem.as_mut_ptr(),
            result.code_buffer.len(),
        );
        let mem = mem.make_exec().unwrap();
        let f: extern "C" fn() -> i32 = ::std::mem::transmute(mem.as_ptr());
        let ans = f();
        println!("{}", ans);
    }
}

fn assemble_function(ctx: &mut Context, func_id: FuncId, func: &Function<X86_64>) {
    let func_label = ctx.new_label();
    ctx.add_func_label(func_id, func_label);

    // fn add_label(id: u64, mut instruction: IcedInst) -> IcedInst {
    //     instruction.set_ip(id);
    //     instruction
    // }

    for block_id in func.layout.block_iter() {
        for inst_id in func.layout.inst_iter(block_id) {
            let inst = func.data.inst_ref(inst_id);
            match inst.data.opcode {
                Opcode::MOVri32 => {
                    let &dst = inst.data.operands[0].data.as_reg();
                    let src = inst.data.operands[1].data.as_i32();
                    ctx.add_instruction(
                        IcedInst::try_with_reg_i32(Code::Mov_r32_imm32, dst.into(), src).unwrap(),
                    );
                }
                Opcode::MOVrr64 => {
                    let &dst = inst.data.operands[0].data.as_reg();
                    let &src = inst.data.operands[1].data.as_reg();
                    ctx.add_instruction(IcedInst::with_reg_reg(
                        Code::Mov_rm64_r64,
                        dst.into(),
                        src.into(),
                    ));
                }
                Opcode::PUSH64 => {
                    let &r = inst.data.operands[0].data.as_reg();
                    ctx.add_instruction(IcedInst::with_reg(Code::Push_r64, r.into()));
                }
                Opcode::POP64 => {
                    let &r = inst.data.operands[0].data.as_reg();
                    ctx.add_instruction(IcedInst::with_reg(Code::Pop_r64, r.into()));
                }
                Opcode::RET => {
                    ctx.add_instruction(IcedInst::with(Code::Retnq));
                }
                _ => todo!(),
            }
        }
    }
}

impl From<Reg> for Register {
    fn from(r: Reg) -> Self {
        match r {
            Reg(0, 0) => Register::EAX,
            Reg(0, 1) => Register::ECX,
            Reg(1, 4) => Register::RSP,
            Reg(1, 5) => Register::RBP,
            _ => todo!(),
        }
    }
}

#[test]
fn jit() {
    use crate::{codegen::lower::compile_module, ir::module::parse_assembly};

    let ir = parse_assembly(
        r#"
define dso_local i32 @main() {
  ret i32 42
}

    "#,
    )
    .unwrap();
    let module = compile_module(crate::codegen::isa::x86_64::X86_64, ir).unwrap();
    println!("{:?}", module);
    let mut ctx = Context::default();
    assemble(&mut ctx, &module);
}
