use crate::asm::{Span, SymbolTable};
use crate::error::BitBoxError;
use crate::mv::Mv;

pub trait Execute {
    fn execute(&mut self, mv: &mut Mv);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub name: String,
    pub span: Span,
    pub def: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R> {
    pub fn map_left<F, T>(self, f: F) -> Either<T, R>
    where
        F: FnOnce(L) -> T,
    {
        match self {
            Either::Left(l) => Either::Left(f(l)),
            Either::Right(r) => Either::Right(r),
        }
    }

    pub fn map_right<F, T>(self, f: F) -> Either<L, T>
    where
        F: FnOnce(R) -> T,
    {
        match self {
            Either::Left(l) => Either::Left(l),
            Either::Right(r) => Either::Right(f(r)),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register {
    R0 = 0,
    R1 = 1,
    R2 = 2,
    R3 = 3,
    R4 = 4,
    R5 = 5,
    R6 = 6,
    R7 = 7,
    R8 = 8,
    R9 = 9,
    R10 = 10,
    R11 = 11,
    R12 = 12,
    R13 = 13,
    R14 = 14,
    R15 = 15,
    R16 = 16,
    R17 = 17,
    R18 = 18,
    R19 = 19,
    R20 = 20,
    R21 = 21,
    R22 = 22,
    R23 = 23,
    R24 = 24,
    R25 = 25,
    R26 = 26,
    R27 = 27,
    R28 = 28,
    R29 = 29,
    R30 = 30,
    R31 = 31,
}

impl TryFrom<(u8, Span)> for Register {
    type Error = BitBoxError;

    fn try_from((value, span): (u8, Span)) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::R0),
            1 => Ok(Self::R1),
            2 => Ok(Self::R2),
            3 => Ok(Self::R3),
            4 => Ok(Self::R4),
            5 => Ok(Self::R5),
            6 => Ok(Self::R6),
            7 => Ok(Self::R7),
            8 => Ok(Self::R8),
            9 => Ok(Self::R9),
            10 => Ok(Self::R10),
            11 => Ok(Self::R11),
            12 => Ok(Self::R12),
            13 => Ok(Self::R13),
            14 => Ok(Self::R14),
            15 => Ok(Self::R15),
            16 => Ok(Self::R16),
            17 => Ok(Self::R17),
            18 => Ok(Self::R18),
            19 => Ok(Self::R19),
            20 => Ok(Self::R20),
            21 => Ok(Self::R21),
            22 => Ok(Self::R22),
            23 => Ok(Self::R23),
            24 => Ok(Self::R24),
            25 => Ok(Self::R25),
            26 => Ok(Self::R26),
            27 => Ok(Self::R27),
            28 => Ok(Self::R28),
            29 => Ok(Self::R29),
            30 => Ok(Self::R30),
            31 => Ok(Self::R31),
            _ => Err(BitBoxError::RegisterOutOfBounds(value, span)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    Load,
    Push,
    Pop,
    Add,
    Inc,
    Eq,
    Jne,
    Hult,
    PrintReg,
}

impl TryFrom<u8> for Opcode {
    type Error = BitBoxError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Opcode::Load),
            1 => Ok(Opcode::Push),
            2 => Ok(Opcode::Pop),
            3 => Ok(Opcode::Add),
            4 => Ok(Opcode::Inc),
            5 => Ok(Opcode::Eq),
            6 => Ok(Opcode::Jne),
            7 => Ok(Opcode::Hult),
            8 => Ok(Opcode::PrintReg),
            _ => Err(BitBoxError::InvalidOpcode(value)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    /// max 128
    U(u8),
    /// max 128
    I(u8),
    Void,
}

impl TryFrom<u8> for Type {
    type Error = BitBoxError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value == 0 {
            return Ok(Type::Void);
        }
        let signned = value & 0b1000_0000 == 0;
        if signned {
            Ok(Type::U(value & 0b0111_1111))
        } else {
            Ok(Type::I(value & 0b0111_1111))
        }
    }
}

impl Type {
    pub fn as_u8(&self) -> u8 {
        match self {
            Type::U(num) => *num,
            Type::I(num) => 0b1000_0000 | *num,
            Type::Void => 0,
        }
    }
    pub fn bytes(&self) -> u8 {
        match self {
            Type::U(num) => *num,
            Type::I(num) => *num,
            Type::Void => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Imm(pub Vec<u8>);
impl From<u32> for Imm {
    fn from(value: u32) -> Self {
        Self(vec![
            value as u8,
            (value >> 8) as u8,
            (value >> 16) as u8,
            (value >> 24) as u8,
        ])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Data {
    NoArgs,
    Reg1(Register),
    Reg2(Register, Register),
    Reg3(Register, Register, Register),
    Imm(Register, Imm),
    RegLabel(Register, Register, Either<Label, u32>),
}

impl Data {
    pub fn to_bytes(&self, symbol_table: &SymbolTable) -> Vec<u8> {
        match self {
            Self::NoArgs => vec![],
            Self::Reg1(reg) => vec![*reg as u8],
            Self::Reg2(reg1, reg2) => vec![*reg1 as u8, *reg2 as u8],
            Self::Reg3(reg1, reg2, reg3) => vec![*reg1 as u8, *reg2 as u8, *reg3 as u8],
            Self::Imm(reg, imm) => vec![*reg as u8].into_iter().chain(imm.0.clone()).collect(),
            // TODO: remove unwrap
            Self::RegLabel(lhs, rhs, Either::Left(Label { name, .. })) => {
                vec![*lhs as u8, *rhs as u8]
                    .into_iter()
                    .chain(symbol_table.get(name).unwrap().to_le_bytes().to_vec())
                    .collect()
            }
            Self::RegLabel(lhs, rhs, Either::Right(value)) => vec![*lhs as u8, *rhs as u8]
                .into_iter()
                .chain(value.to_le_bytes().to_vec())
                .collect(),
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Self::NoArgs => 0,
            Self::Reg1(_) => 1,
            Self::Reg2(_, _) => 2,
            Self::Reg3(_, _, _) => 3,
            Self::Imm(_, imm) => 1 + imm.0.len(),
            Self::RegLabel(..) => 6,
        }
    }
}

/// Represents an instruction
/// Opcode = u8
/// type = u8
/// data = u16 or as many u16 chunks as needed
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub opcode: Opcode,
    pub r#type: Type,
    pub data: Data,
}

impl Instruction {
    pub fn to_bytes(&self, symbol_table: &SymbolTable) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.opcode as u8);
        bytes.push(self.r#type.as_u8());
        bytes.extend(self.data.to_bytes(symbol_table));
        bytes
    }

    pub fn size(&self) -> u32 {
        let opcode = 1;
        let type_ = 1;
        let data = self.data.size() as u32;
        opcode + type_ + data
    }
}

// TODO: Move this into the mv struct
impl Execute for Instruction {
    fn execute(&mut self, mv: &mut Mv) {
        match self.opcode {
            Opcode::Load => match &self.data {
                Data::Imm(reg, Imm(value)) => {
                    let size = self.r#type.bytes();
                    // debug_assert_eq!(value.len(), size as usize);
                    match size {
                        8 => mv.set_regester(*reg as u8, value[0] as u32),
                        16 => mv.set_regester(
                            *reg as u8,
                            u16::from_le_bytes(value[0..2].try_into().expect("Not enough bytes"))
                                as u32,
                        ),
                        32 => mv.set_regester(
                            *reg as u8,
                            u32::from_le_bytes(value[0..4].try_into().expect("Not enough bytes")),
                        ),
                        _ => unimplemented!("Unimplemented size: {}", size),
                    }
                }
                _ => unimplemented!("Load with two registers not implemented"),
            },
            Opcode::Push => match self.data {
                Data::Reg1(reg) => {
                    let value = *mv.get_regester(reg as u8);
                    mv.push_to_stack(value);
                }
                _ => unreachable!("Push with two registers not implemented"),
            },
            Opcode::Pop => match self.data {
                Data::Reg1(reg) => {
                    let value = mv.pop_from_stack();
                    mv.set_regester(reg as u8, value);
                }
                _ => unreachable!("Push with two registers not implemented"),
            },
            Opcode::Add => match self.data {
                Data::Reg3(des, reg_lhs, reg_rhs) => {
                    let lhs = mv.get_regester(reg_lhs as u8);
                    let rhs = mv.get_regester(reg_rhs as u8);
                    mv.set_regester(des as u8, lhs + rhs);
                }
                _ => unreachable!(),
            },
            Opcode::Inc => match self.data {
                Data::Reg1(reg) => {
                    let value = mv.get_regester(reg as u8);
                    mv.set_regester(reg as u8, value + 1);
                }
                _ => unreachable!(),
            },
            Opcode::Jne => match self.data {
                Data::RegLabel(lhs, rhs, Either::Right(label)) => {
                    let lhs_value = mv.get_regester(lhs as u8);
                    let rhs_value = mv.get_regester(rhs as u8);
                    if lhs_value == rhs_value {
                        return;
                    }
                    mv.pc = label as usize;
                }
                _ => unreachable!(),
            },
            Opcode::Eq => match self.data {
                Data::Reg3(des, reg_lhs, reg_rhs) => {
                    let lhs = mv.get_regester(reg_lhs as u8);
                    let rhs = mv.get_regester(reg_rhs as u8);
                    mv.set_regester(des as u8, (lhs == rhs) as u32);
                }
                _ => unreachable!(),
            },
            Opcode::Hult => mv.running = false,
            Opcode::PrintReg => match self.data {
                Data::Reg1(reg) => {
                    let value = mv.get_regester(reg as u8);
                    println!("{}", value);
                }
                _ => unreachable!(),
            },
        }
    }
}

// impl TryFrom<(u8, u8, u8, u8)> for Instruction {
//     type Error = BitBoxError;
//
//     fn try_from((a, b, c, d): (u8, u8, u8, u8)) -> Result<Self, Self::Error> {
//         match a {
//             0 => Ok(Instruction::LoadInt(b, c, d)),
//             1 => Ok(Instruction::Add(b, c, d)),
//             2 => Ok(Instruction::Hult),
//             _ => Err(BitBoxError::InvalidInstruction(a)),
//         }
//     }
// }
//
// impl Execute for Instruction {
//     fn execute(&mut self, mv: &mut Mv) {
//         match self {
//             Instruction::LoadInt(reg, lower_value, upper_value) => {
//                 LoadInt::from((*reg, *lower_value, *upper_value)).execute(mv)
//             }
//             Instruction::Add(des, reg_lhs, reg_rhs) => {
//                 Add::from((*des, *reg_lhs, *reg_rhs)).execute(mv)
//             }
//             Instruction::Hult => Hult.execute(mv),
//         }
//     }
// }
//
// struct LoadInt {
//     pub reg: u8,
//     pub lower_value: u8,
//     pub upper_value: u8,
// }
//
// impl From<(u8, u8, u8)> for LoadInt {
//     fn from((reg, lower_value, upper_value): (u8, u8, u8)) -> Self {
//         Self {
//             reg,
//             lower_value,
//             upper_value,
//         }
//     }
// }
//
// impl Execute for LoadInt {
//     fn execute(&mut self, mv: &mut Mv) {
//         let Self {
//             reg,
//             lower_value: b1,
//             upper_value: b2,
//         } = *self;
//         let value = (b1 as u16) << 8 | b2 as u16;
//         mv.set_regester(reg, value as u32);
//     }
// }
//
// struct Add {
//     pub des: u8,
//     pub reg_lhs: u8,
//     pub reg_rhs: u8,
// }
//
// impl From<(u8, u8, u8)> for Add {
//     fn from((des, reg_lhs, reg_rhs): (u8, u8, u8)) -> Self {
//         Self {
//             des,
//             reg_lhs,
//             reg_rhs,
//         }
//     }
// }
//
// impl Execute for Add {
//     fn execute(&mut self, mv: &mut Mv) {
//         let Self {
//             des,
//             reg_lhs,
//             reg_rhs,
//         } = *self;
//         let lhs = mv.get_regester(reg_lhs);
//         let rhs = mv.get_regester(reg_rhs);
//         mv.set_regester(des, lhs + rhs);
//     }
// }
//
// struct Hult;
// impl Execute for Hult {
//     fn execute(&mut self, mv: &mut Mv) {
//         mv.running = false;
//     }
// }
