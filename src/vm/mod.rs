#[cfg(test)]
mod tests;
use crate::asm::{Header, Span};
use crate::error::BitBoxError;
use crate::instructions::{Data, Execute, Imm, Instruction, Opcode, Register, Type};
use crate::utils::Either;
use anyhow::{bail, Result};

pub struct Vm {
    pub program: Vec<u8>,
    pub regesters: Vec<u64>,
    pub stack: Vec<u64>,
    pub heap: Vec<u8>,
    pub pc: usize,
    pub running: bool,
}

// Public implementation
impl Vm {
    pub const REGESTER_COUNT: usize = 32;
    pub fn new(program: Vec<u8>) -> Result<Self> {
        let entry_point = &program[Header::ENTRY_OFFSET..Header::ENTRY_OFFSET + 4];
        let pc = usize::from_le_bytes([
            entry_point[0],
            entry_point[1],
            entry_point[2],
            entry_point[3],
            0,
            0,
            0,
            0,
        ]);

        Ok(Self {
            program,
            regesters: vec![0; Self::REGESTER_COUNT],
            stack: Vec::new(),
            heap: Vec::new(),
            pc,
            running: true,
        })
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        let length = args.len() as u64;
        self.set_regester(0, length);

        let mut ptr = 0u64;
        for arg in args.into_iter() {
            let len = arg.len() as u64;
            self.stack.push((len << 32) | ptr);
            for c in arg.chars() {
                self.heap.push(c as u8);
            }

            ptr += len;
        }

        self.stack.reverse();

        self
    }

    pub fn get_regester(&self, reg: u8) -> &u64 {
        &self.regesters[reg as usize]
    }

    pub fn set_regester(&mut self, reg: u8, value: u64) {
        self.regesters[reg as usize] = value;
    }

    pub fn push_to_stack(&mut self, value: u64) {
        self.stack.push(value);
    }

    pub fn pop_from_stack(&mut self) -> u64 {
        self.stack.pop().unwrap()
    }

    pub fn set_heap_u8(&mut self, dest: u64, value: u8) {
        self.heap[dest as usize] = value
    }

    pub fn set_heap_u16(&mut self, dest: u64, value: u16) {
        let bytes = value.to_le_bytes();
        self.heap[dest as usize] = bytes[0];
        self.heap[dest as usize + 1] = bytes[1];
    }

    pub fn set_heap_u32(&mut self, dest: u64, value: u32) {
        let bytes = value.to_le_bytes();
        self.heap[dest as usize] = bytes[0];
        self.heap[dest as usize + 1] = bytes[1];
        self.heap[dest as usize + 2] = bytes[2];
        self.heap[dest as usize + 3] = bytes[3];
    }

    pub fn set_heap_u64(&mut self, dest: u64, value: u64) {
        let bytes = value.to_le_bytes();
        self.heap[dest as usize] = bytes[0];
        self.heap[dest as usize + 1] = bytes[1];
        self.heap[dest as usize + 2] = bytes[2];
        self.heap[dest as usize + 3] = bytes[3];
        self.heap[dest as usize + 4] = bytes[4];
        self.heap[dest as usize + 5] = bytes[5];
        self.heap[dest as usize + 6] = bytes[6];
        self.heap[dest as usize + 7] = bytes[7];
    }

    pub fn execute(&mut self) -> Result<()> {
        let opcode: Opcode = self.get_next_byte().try_into()?;
        match opcode {
            Opcode::Load => self.opcode_1reg_imm(Opcode::Load)?,
            Opcode::Store => self.opcode_2reg(Opcode::Store)?,
            Opcode::Copy => self.opcode_2reg(Opcode::Copy)?,
            Opcode::Aloc => self.opcode_1reg(Opcode::Aloc)?,
            Opcode::Push => self.opcode_1reg(Opcode::Push)?,
            Opcode::Pop => self.opcode_1reg(Opcode::Pop)?,
            Opcode::Add => self.opcode_3reg(Opcode::Add)?,
            Opcode::Sub => self.opcode_3reg(Opcode::Sub)?,
            Opcode::Div => self.opcode_3reg(Opcode::Div)?,
            Opcode::Mul => self.opcode_3reg(Opcode::Mul)?,
            Opcode::Inc => self.opcode_1reg(Opcode::Inc)?,
            Opcode::Eq => self.opcode_3reg(Opcode::Eq)?,
            Opcode::Jne => self.opcode_2reg_label(Opcode::Jne)?,
            Opcode::Hult => self.opcode_noargs(Opcode::Hult)?,
            Opcode::PrintReg => self.opcode_1reg(Opcode::PrintReg)?,
            Opcode::Call => self.opcode_label(Opcode::Call)?,
            Opcode::And => self.opcode_3reg(Opcode::And)?,
            Opcode::Or => self.opcode_3reg(Opcode::Or)?,
            Opcode::Shr => self.opcode_3reg(Opcode::Shr)?,
            Opcode::Return => self.opcode_noargs(Opcode::Return)?,
            Opcode::Syscall => self.opcode_noargs(Opcode::Syscall)?,
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        while self.running {
            self.execute()?;
        }
        Ok(())
    }
}
// Private implementation
impl Vm {
    fn get_next_byte(&mut self) -> u8 {
        let pc = self.pc;
        self.pc += 1;
        self.program[pc]
    }

    fn opcode_noargs(&mut self, opcode: Opcode) -> Result<()> {
        let Type::Void: Type = self.get_next_byte().try_into()? else {
            panic!("Error: expected void");
        };
        Instruction {
            opcode,
            r#type: Type::Void,
            data: Data::NoArgs,
        }
        .execute(self)?;
        Ok(())
    }

    fn opcode_label(&mut self, opcode: Opcode) -> Result<()> {
        let r#type: Type = self.get_next_byte().try_into()?;

        let address = u32::from_le_bytes([
            self.get_next_byte(),
            self.get_next_byte(),
            self.get_next_byte(),
            self.get_next_byte(),
        ]);

        Instruction {
            opcode,
            r#type,
            data: Data::Label(Either::Right(address)),
        }
        .execute(self)?;
        Ok(())
    }

    fn opcode_1reg(&mut self, opcode: Opcode) -> Result<()> {
        let r#type: Type = self.get_next_byte().try_into()?;
        let reg: Register = (self.get_next_byte(), Span::default()).try_into()?;
        Instruction {
            opcode,
            r#type,
            data: Data::Reg1(reg),
        }
        .execute(self)?;
        Ok(())
    }

    fn opcode_2reg(&mut self, opcode: Opcode) -> Result<()> {
        let r#type: Type = self.get_next_byte().try_into()?;
        let reg1: Register = (self.get_next_byte(), Span::default()).try_into()?;
        let reg2: Register = (self.get_next_byte(), Span::default()).try_into()?;
        Instruction {
            opcode,
            r#type,
            data: Data::Reg2(reg1, reg2),
        }
        .execute(self)?;
        Ok(())
    }

    fn opcode_3reg(&mut self, opcode: Opcode) -> Result<()> {
        let r#type: Type = self.get_next_byte().try_into()?;
        let reg1: Register = (self.get_next_byte(), Span::default()).try_into()?;
        let reg2: Register = (self.get_next_byte(), Span::default()).try_into()?;
        let reg3: Register = (self.get_next_byte(), Span::default()).try_into()?;
        Instruction {
            opcode,
            r#type,
            data: Data::Reg3(reg1, reg2, reg3),
        }
        .execute(self)?;
        Ok(())
    }

    fn opcode_1reg_imm(&mut self, opcode: Opcode) -> Result<()> {
        let r#type: Type = self.get_next_byte().try_into()?;
        let reg: Register = (self.get_next_byte(), Span::default()).try_into()?;

        let mut data = Vec::new();
        // TODO: CLEAN THIS UP
        match r#type {
            Type::U(8) => data.push(self.get_next_byte()),
            Type::U(16) => data.extend_from_slice(&[self.get_next_byte(), self.get_next_byte()]),
            Type::U(32) => data.extend_from_slice(&[
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
            ]),
            Type::U(64) => data.extend_from_slice(&[
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
            ]),
            Type::U(128) => data.extend_from_slice(&[
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
            ]),
            Type::I(8) => todo!(),
            Type::I(16) => todo!(),
            Type::I(32) => todo!(),
            Type::I(64) => todo!(),
            Type::I(128) => todo!(),
            Type::Void => {}
            _ => unreachable!("Unimplemented type: {:?}", r#type),
        }

        Instruction {
            opcode,
            r#type,
            data: Data::Imm(reg, Imm(data)),
        }
        .execute(self)?;
        Ok(())
    }

    fn opcode_2reg_label(&mut self, opcode: Opcode) -> Result<()> {
        let r#type: Type = self.get_next_byte().try_into()?;
        let lhs: Register = (self.get_next_byte(), Span::default()).try_into()?;
        let rhs: Register = (self.get_next_byte(), Span::default()).try_into()?;
        let label = u32::from_le_bytes([
            self.get_next_byte(),
            self.get_next_byte(),
            self.get_next_byte(),
            self.get_next_byte(),
        ]);
        Instruction {
            opcode,
            r#type,
            data: Data::Reg2Label(lhs, rhs, Either::Right(label)),
        }
        .execute(self)?;
        Ok(())
    }
}

impl Execute for Instruction {
    fn execute(&mut self, vm: &mut Vm) -> Result<()> {
        match self.opcode {
            Opcode::Load => match &self.data {
                Data::Imm(reg, Imm(value)) => {
                    let size = self.r#type.bytes();
                    match size {
                        8 => vm.set_regester(*reg as u8, value[0] as u64),
                        16 => vm.set_regester(
                            *reg as u8,
                            u16::from_le_bytes(value[0..2].try_into().expect("Not enough bytes"))
                                as u64,
                        ),
                        32 => vm.set_regester(
                            *reg as u8,
                            u32::from_le_bytes(value[0..4].try_into().expect("Not enough bytes"))
                                as u64,
                        ),
                        64 => vm.set_regester(
                            *reg as u8,
                            u64::from_le_bytes(value[0..8].try_into().expect("Not enough bytes")),
                        ),
                        _ => unimplemented!(
                            "Error for Load instruction {:?},\n{:?},\n{:#?}",
                            size,
                            vm.pc,
                            vm.program
                        ),
                    }
                    Ok(())
                }
                _ => unimplemented!(
                    "Error for Load instruction: Load with two registers not implemented"
                ),
            },
            Opcode::Store => match &self.data {
                Data::Reg2(reg1, reg2) => {
                    let des = *vm.get_regester(*reg1 as u8);
                    let value = *vm.get_regester(*reg2 as u8);
                    match self.r#type {
                        Type::U(8) => vm.set_heap_u8(des, value as u8),
                        Type::U(16) => vm.set_heap_u16(des, value as u16),
                        Type::U(32) => vm.set_heap_u32(des, value as u32),
                        Type::U(64) => vm.set_heap_u64(des, value),
                        Type::Void => todo!(),
                        _ => unimplemented!("Error for Store instruction"),
                    }
                    Ok(())
                }
                _ => unimplemented!(
                    "Error for Store instruction: Store with two registers not implemented"
                ),
            },
            Opcode::Copy => match self.data {
                Data::Reg2(des, src) => {
                    // TODO: Copy of the type size could be valuable.
                    let value = *vm.get_regester(src as u8);
                    vm.set_regester(des as u8, value);
                    Ok(())
                }
                _ => unreachable!("Error for Copy instruction"),
            },
            Opcode::Aloc => match self.data {
                Data::Reg1(reg) => {
                    let value = *vm.get_regester(reg as u8) as usize;
                    let current_heap_size = vm.heap.len();
                    vm.heap
                        .resize_with(current_heap_size + value, Default::default);
                    Ok(())
                }
                _ => unreachable!("Error for Aloc instruction"),
            },
            Opcode::Push => match self.data {
                Data::Reg1(reg) => {
                    let value = *vm.get_regester(reg as u8);
                    vm.push_to_stack(value);
                    Ok(())
                }
                _ => unreachable!("Error for Push instruction"),
            },
            Opcode::Pop => match self.data {
                Data::Reg1(reg) => {
                    let value = vm.pop_from_stack();
                    vm.set_regester(reg as u8, value);
                    Ok(())
                }
                _ => unreachable!("Error for Pop instruction"),
            },
            Opcode::Add => match self.data {
                Data::Reg3(des, reg_lhs, reg_rhs) => {
                    let lhs = vm.get_regester(reg_lhs as u8);
                    let rhs = vm.get_regester(reg_rhs as u8);
                    vm.set_regester(des as u8, lhs + rhs);
                    Ok(())
                }
                _ => unreachable!("Error for Add instruction"),
            },
            Opcode::Sub => match self.data {
                Data::Reg3(des, reg_lhs, reg_rhs) => {
                    let lhs = vm.get_regester(reg_lhs as u8);
                    let rhs = vm.get_regester(reg_rhs as u8);
                    vm.set_regester(des as u8, lhs - rhs);
                    Ok(())
                }
                _ => unreachable!("Error for Sub instruction"),
            },
            Opcode::Div => match self.data {
                Data::Reg3(des, reg_lhs, reg_rhs) => {
                    let lhs = vm.get_regester(reg_lhs as u8);
                    let rhs = vm.get_regester(reg_rhs as u8);
                    vm.set_regester(des as u8, lhs / rhs);
                    Ok(())
                }
                _ => unreachable!("Error for Div instruction"),
            },
            Opcode::Mul => match self.data {
                Data::Reg3(des, reg_lhs, reg_rhs) => {
                    let lhs = vm.get_regester(reg_lhs as u8);
                    let rhs = vm.get_regester(reg_rhs as u8);
                    vm.set_regester(des as u8, lhs * rhs);
                    Ok(())
                }
                _ => unreachable!("Error for Mul instruction"),
            },
            Opcode::Inc => match self.data {
                Data::Reg1(reg) => {
                    let value = vm.get_regester(reg as u8);
                    vm.set_regester(reg as u8, value + 1);
                    Ok(())
                }
                _ => unreachable!("Error for Inc instruction"),
            },
            Opcode::Jne => match self.data {
                Data::Reg2Label(lhs, rhs, Either::Right(label)) => {
                    let lhs_value = vm.get_regester(lhs as u8);
                    let rhs_value = vm.get_regester(rhs as u8);
                    if lhs_value == rhs_value {
                        return Ok(());
                    }
                    vm.pc = label as usize;
                    Ok(())
                }
                _ => unreachable!("Error for Jne instruction"),
            },
            Opcode::Eq => match self.data {
                Data::Reg3(des, reg_lhs, reg_rhs) => {
                    let lhs = vm.get_regester(reg_lhs as u8);
                    let rhs = vm.get_regester(reg_rhs as u8);
                    vm.set_regester(des as u8, (lhs == rhs) as u64);
                    Ok(())
                }
                _ => unreachable!("Error for Eq instruction"),
            },
            Opcode::Hult => {
                vm.running = false;
                Ok(())
            }
            Opcode::PrintReg => match self.data {
                Data::Reg1(reg) => {
                    let value = vm.get_regester(reg as u8);
                    println!("{}", value);
                    Ok(())
                }
                _ => unreachable!("Error for PrintReg instruction"),
            },
            Opcode::Call => match self.data {
                Data::Label(Either::Right(value)) => {
                    // Prologue
                    vm.stack.push(vm.pc as u64);
                    vm.pc = value as usize;
                    Ok(())
                }
                _ => unreachable!("Error for Call instruction"),
            },
            Opcode::And => match self.data {
                Data::Reg3(des, reg_lhs, reg_rhs) => {
                    let lhs = vm.get_regester(reg_lhs as u8);
                    let rhs = vm.get_regester(reg_rhs as u8);
                    vm.set_regester(des as u8, lhs & rhs);
                    Ok(())
                }
                _ => unreachable!("Error for And instruction"),
            },
            Opcode::Or => match self.data {
                Data::Reg3(des, reg_lhs, reg_rhs) => {
                    let lhs = vm.get_regester(reg_lhs as u8);
                    let rhs = vm.get_regester(reg_rhs as u8);
                    vm.set_regester(des as u8, lhs | rhs);
                    Ok(())
                }
                _ => unreachable!("Error for Or instruction"),
            },
            Opcode::Shr => match self.data {
                Data::Reg3(des, reg_lhs, reg_rhs) => {
                    let lhs = vm.get_regester(reg_lhs as u8);
                    let rhs = vm.get_regester(reg_rhs as u8);
                    vm.set_regester(des as u8, lhs >> rhs);
                    Ok(())
                }
                _ => unreachable!("Error for Or instruction"),
            },
            Opcode::Return => {
                // Epilogue
                let Some(value) = vm.stack.pop() else {
                    bail!(BitBoxError::StackUnderflow);
                };
                vm.pc = value as usize;
                Ok(())
            }
            Opcode::Syscall => match vm.get_regester(Register::R0 as u8) {
                // Write
                0 => {
                    use std::io::Write;
                    let ptr = *vm.get_regester(Register::R1 as u8) as usize;
                    let length = *vm.get_regester(Register::R2 as u8) as usize;
                    let _static = if *vm.get_regester(Register::R3 as u8) == 1 {
                        true
                    } else {
                        false
                    };

                    let data = &vm.heap[ptr..(ptr + length)];
                    print!("{}", String::from_utf8_lossy(data));
                    std::io::stdout().flush()?;
                    Ok(())
                }
                _ => unreachable!("Error for Syscall instruction"),
            },
        }
    }
}
