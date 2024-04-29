use crate::asm::Header;
use crate::instructions::{Execute, Instruction};
use anyhow::Result;

pub struct Mv {
    program: Vec<u8>,
    regesters: Vec<u32>,
    pc: usize,
    pub running: bool,
}

// Public implementation
impl Mv {
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
            regesters: vec![0; 32],
            pc,
            running: true,
        })
    }

    pub fn get_regester(&self, reg: u8) -> &u32 {
        &self.regesters[reg as usize]
    }

    pub fn set_regester(&mut self, reg: u8, value: u32) {
        self.regesters[reg as usize] = value;
    }

    pub fn execute(&mut self) -> Result<()> {
        let a: u8 = self.get_next_byte().into();
        let b: u8 = self.get_next_byte();
        let c: u8 = self.get_next_byte();
        let d: u8 = self.get_next_byte();
        let mut instruction = Instruction::try_from((a, b, c, d))?;
        instruction.execute(self);
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        while self.running {
            self.execute()?;
        }
        println!("{:?}", self.regesters);
        Ok(())
    }
}
// Private implementation
impl Mv {
    fn get_next_byte(&mut self) -> u8 {
        let pc = self.pc;
        self.pc += 1;
        self.program[pc]
    }
}
