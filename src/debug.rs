use crate::chunk::{Chunk, OpCode};

impl Chunk {
    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);

        let mut offset = 0;
        while offset < self.code().len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:04} ", offset);

        if offset > 0 && self.lines()[offset - 1] == self.lines()[offset] {
            print!("   | ");
        } else {
            print!("{:4} ", self.lines()[offset]);
        }

        let instruction = OpCode::from(self.code()[offset]);

        fn simple_instruction(name: &str, offset: usize) -> usize {
            println!("{}", name);
            offset + 1
        }

        fn constant_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
            let constant = chunk.code()[offset + 1];
            println!("{:<16} {:4} '{}'", name, constant, chunk.constants().values()[constant as usize]);
            offset + 2
        }

        match instruction {
            OpCode::Return => simple_instruction("OP_RETURN", offset),
            OpCode::Constant => constant_instruction("OP_CONSTANT", self, offset),
            _ => {
                println!("Unknown opcode: {:?}", instruction);
                offset + 1
            },
        }
    }
}
