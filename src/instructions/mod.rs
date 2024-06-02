use crate::asm::{Span, SymbolTable};
use crate::error::BitBoxError;
use crate::utils::Either;
use crate::vm::Vm;
use anyhow::Result;

pub trait Execute {
    fn execute(&mut self, mv: &mut Vm) -> Result<()>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub name: String,
    pub span: Span,
    pub def: bool,
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
    Store,
    Aloc,
    Push,
    Pop,
    Add,
    Sub,
    Div,
    Mul,
    Inc,
    Eq,
    Jne,
    Hult,
    PrintReg,
    Call,
    And,
    Or,
    Return,
    Syscall,
}

impl TryFrom<u8> for Opcode {
    type Error = BitBoxError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Opcode::Load),
            1 => Ok(Opcode::Store),
            2 => Ok(Opcode::Aloc),
            3 => Ok(Opcode::Push),
            4 => Ok(Opcode::Pop),
            5 => Ok(Opcode::Add),
            6 => Ok(Opcode::Sub),
            7 => Ok(Opcode::Div),
            8 => Ok(Opcode::Mul),
            9 => Ok(Opcode::Inc),
            10 => Ok(Opcode::Eq),
            11 => Ok(Opcode::Jne),
            12 => Ok(Opcode::Hult),
            13 => Ok(Opcode::PrintReg),
            14 => Ok(Opcode::Call),
            15 => Ok(Opcode::And),
            16 => Ok(Opcode::Or),
            17 => Ok(Opcode::Return),
            18 => Ok(Opcode::Syscall),
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
impl From<u8> for Imm {
    fn from(value: u8) -> Self {
        Self(vec![value as u8])
    }
}

impl From<u16> for Imm {
    fn from(value: u16) -> Self {
        Self(vec![value as u8, (value >> 8) as u8])
    }
}

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

impl From<u64> for Imm {
    fn from(value: u64) -> Self {
        Self(vec![
            value as u8,
            (value >> 8) as u8,
            (value >> 16) as u8,
            (value >> 24) as u8,
            (value >> 32) as u8,
            (value >> 40) as u8,
            (value >> 48) as u8,
            (value >> 56) as u8,
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
    Label(Either<Label, u32>),
    Reg2Label(Register, Register, Either<Label, u32>),
}

impl Data {
    pub fn to_bytes(&self, symbol_table: &SymbolTable) -> Result<Vec<u8>> {
        match self {
            Self::NoArgs => Ok(vec![]),
            Self::Reg1(reg) => Ok(vec![*reg as u8]),
            Self::Reg2(reg1, reg2) => Ok(vec![*reg1 as u8, *reg2 as u8]),
            Self::Reg3(reg1, reg2, reg3) => Ok(vec![*reg1 as u8, *reg2 as u8, *reg3 as u8]),
            Self::Imm(reg, imm) => Ok(vec![*reg as u8].into_iter().chain(imm.0.clone()).collect()),
            Self::Label(Either::Left(Label { name, span, .. })) => Ok(symbol_table
                .get(name)
                .ok_or(BitBoxError::UnknownLabel(name.clone(), *span))?
                .to_le_bytes()
                .to_vec()),
            Self::Label(Either::Right(value)) => Ok(value.to_le_bytes().to_vec()),
            Self::Reg2Label(lhs, rhs, Either::Left(Label { name, span, .. })) => {
                Ok(vec![*lhs as u8, *rhs as u8]
                    .into_iter()
                    .chain(
                        symbol_table
                            .get(name)
                            .ok_or(BitBoxError::UnknownLabel(name.clone(), *span))?
                            .to_le_bytes()
                            .to_vec(),
                    )
                    .collect())
            }
            Self::Reg2Label(lhs, rhs, Either::Right(value)) => Ok(vec![*lhs as u8, *rhs as u8]
                .into_iter()
                .chain(value.to_le_bytes().to_vec())
                .collect()),
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Self::NoArgs => 0,
            Self::Reg1(_) => 1,
            Self::Reg2(_, _) => 2,
            Self::Reg3(_, _, _) => 3,
            Self::Imm(_, imm) => 1 + imm.0.len(),
            Self::Label(..) => 4,
            Self::Reg2Label(..) => 6,
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
    pub fn to_bytes(&self, symbol_table: &SymbolTable) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        bytes.push(self.opcode as u8);
        bytes.push(self.r#type.as_u8());
        bytes.extend(self.data.to_bytes(symbol_table)?);
        Ok(bytes)
    }

    pub fn size(&self) -> u32 {
        let opcode = 1;
        let type_ = 1;
        let data = self.data.size() as u32;
        opcode + type_ + data
    }
}
