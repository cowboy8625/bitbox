use anyhow::Result;

fn main() -> Result<()> {
    Mv::new(vec![1, 1, 0, 7, 1, 0, 0, 7, 0, 0, 0, 0]).run()
}

trait Execute {
    fn execute(&mut self, mv: &mut Mv, bytes: [u8; 3]);
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    LoadInt,
    Add,
    Hult,
}

impl Execute for Instruction {
    fn execute(&mut self, mv: &mut Mv, bytes: [u8; 3]) {
        match self {
            Instruction::LoadInt => LoadInt.execute(mv, bytes),
            Instruction::Add => Add.execute(mv, bytes),
            Instruction::Hult => Hult.execute(mv, bytes),
        }
    }
}

struct LoadInt;
impl Execute for LoadInt {
    fn execute(&mut self, mv: &mut Mv, [reg, b1, b2]: [u8; 3]) {
        let value = (b1 as u16) << 8 | b2 as u16;
        mv.regesters[reg as usize] = value as u32;
    }
}

struct Hult;
impl Execute for Hult {
    fn execute(&mut self, mv: &mut Mv, _: [u8; 3]) {
        mv.running = false;
    }
}

// ---------------------------------------

struct Mv {
    program: Vec<u8>,
    regesters: Vec<u32>,
    pc: usize,
    running: bool,
}

impl Mv {
    fn new(program: Vec<u8>) -> Self {
        Self {
            program,
            regesters: vec![0; 32],
            pc: 0,
            running: true,
        }
    }

    fn construct_next_instruction(&mut self) -> Box<dyn Instruction> {
        let a = self.get_next_byte();

        match a {
            0 => Box::new(Hult),
            1 => Box::new(LoadInt),
            _ => unimplemented!(),
        }
    }

    fn get_next_byte(&mut self) -> u8 {
        let pc = self.pc;
        self.pc += 1;
        self.program[pc]
    }

    fn execute(&mut self) {
        let mut instruction = self.construct_next_instruction();
        let b = self.get_next_byte();
        let c = self.get_next_byte();
        let d = self.get_next_byte();
        instruction.execute(self, [b, c, d]);
    }

    fn run(&mut self) -> Result<()> {
        while self.running {
            self.execute();
        }
        println!("{:?}", self.regesters);
        Ok(())
    }
}
