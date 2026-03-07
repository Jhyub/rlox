use crate::chunk::{Chunk, OpCode};

impl Chunk {
    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);

        let mut offset = 0;
        while offset < self.code().len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {
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
            OpCode::Nil => simple_instruction("OP_NIL", offset),
            OpCode::True => simple_instruction("OP_TRUE", offset),
            OpCode::False => simple_instruction("OP_FALSE", offset),
            OpCode::Equal => simple_instruction("OP_EQUAL", offset),
            OpCode::Greater => simple_instruction("OP_GREATER", offset),
            OpCode::Less => simple_instruction("OP_LESS", offset),
            OpCode::Add => simple_instruction("OP_ADD", offset),
            OpCode::Subtract => simple_instruction("OP_SUBTRACT", offset),
            OpCode::Multiply => simple_instruction("OP_MULTIPLY", offset),
            OpCode::Divide => simple_instruction("OP_DIVIDE", offset),
            OpCode::Negate => simple_instruction("OP_NEGATE", offset),
            OpCode::Not => simple_instruction("OP_NOT", offset),
            OpCode::Constant => constant_instruction("OP_CONSTANT", self, offset),
            OpCode::Print => simple_instruction("OP_PRINT", offset),
            OpCode::Pop => simple_instruction("OP_POP", offset),
            OpCode::DefineGlobal => constant_instruction("OP_DEFINE_GLOBAL", self, offset),
            OpCode::GetGlobal => constant_instruction("OP_GET_GLOBAL", self, offset),
            OpCode::SetGlobal => constant_instruction("OP_SET_GLOBAL", self, offset),
            /*
            _ => {
                println!("Unknown opcode: {:?}", instruction);
                offset + 1
            },
            */
        }
    }
}
