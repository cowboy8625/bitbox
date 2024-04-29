use crate::error::BitBoxError;
use crate::mv::Mv;

pub trait Execute {
    fn execute(&mut self, mv: &mut Mv);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    LoadInt(u8, u8, u8),
    Add(u8, u8, u8),
    Hult,
}

impl Instruction {
    const LOAD_INT: u8 = 0;
    const ADD: u8 = 1;
    const HULT: u8 = 2;
    pub fn to_bytes(&self) -> [u8; 4] {
        match self {
            Instruction::LoadInt(a, b, c) => [Self::LOAD_INT, *a, *b, *c],
            Instruction::Add(a, b, c) => [Self::ADD, *a, *b, *c],
            Instruction::Hult => [Self::HULT, 0, 0, 0],
        }
    }
}

impl TryFrom<(u8, u8, u8, u8)> for Instruction {
    type Error = BitBoxError;

    fn try_from((a, b, c, d): (u8, u8, u8, u8)) -> Result<Self, Self::Error> {
        match a {
            0 => Ok(Instruction::LoadInt(b, c, d)),
            1 => Ok(Instruction::Add(b, c, d)),
            2 => Ok(Instruction::Hult),
            _ => Err(BitBoxError::InvalidInstruction(a)),
        }
    }
}

impl Execute for Instruction {
    fn execute(&mut self, mv: &mut Mv) {
        match self {
            Instruction::LoadInt(reg, lower_value, upper_value) => {
                LoadInt::from((*reg, *lower_value, *upper_value)).execute(mv)
            }
            Instruction::Add(des, reg_lhs, reg_rhs) => {
                Add::from((*des, *reg_lhs, *reg_rhs)).execute(mv)
            }
            Instruction::Hult => Hult.execute(mv),
        }
    }
}

struct LoadInt {
    pub reg: u8,
    pub lower_value: u8,
    pub upper_value: u8,
}

impl From<(u8, u8, u8)> for LoadInt {
    fn from((reg, lower_value, upper_value): (u8, u8, u8)) -> Self {
        Self {
            reg,
            lower_value,
            upper_value,
        }
    }
}

impl Execute for LoadInt {
    fn execute(&mut self, mv: &mut Mv) {
        let Self {
            reg,
            lower_value: b1,
            upper_value: b2,
        } = *self;
        let value = (b1 as u16) << 8 | b2 as u16;
        mv.set_regester(reg, value as u32);
    }
}

struct Add {
    pub des: u8,
    pub reg_lhs: u8,
    pub reg_rhs: u8,
}

impl From<(u8, u8, u8)> for Add {
    fn from((des, reg_lhs, reg_rhs): (u8, u8, u8)) -> Self {
        Self {
            des,
            reg_lhs,
            reg_rhs,
        }
    }
}

impl Execute for Add {
    fn execute(&mut self, mv: &mut Mv) {
        let Self {
            des,
            reg_lhs,
            reg_rhs,
        } = *self;
        let lhs = mv.get_regester(reg_lhs);
        let rhs = mv.get_regester(reg_rhs);
        mv.set_regester(des, lhs + rhs);
    }
}

struct Hult;
impl Execute for Hult {
    fn execute(&mut self, mv: &mut Mv) {
        mv.running = false;
    }
}
