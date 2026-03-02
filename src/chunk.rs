use crate::value::{Value, ValueArray};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum OpCode {
    Constant,
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Return,
}

impl From<u8> for OpCode {
    fn from(byte: u8) -> Self {
        match byte {
            0 => OpCode::Constant,
            1 => OpCode::Add,
            2 => OpCode::Subtract,
            3 => OpCode::Multiply,
            4 => OpCode::Divide,
            5 => OpCode::Negate,
            6 => OpCode::Return,
            _ => panic!("Invalid opcode: {}", byte),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Chunk {
    code: Vec<u8>,
    lines: Vec<i32>,
    constants: ValueArray,
}

impl Chunk {
    pub fn code(&self) -> &[u8] {
        &self.code
    }

    pub fn lines(&self) -> &[i32] {
        &self.lines
    }

    pub fn new() -> Self {
        Self { 
            code: Vec::new(),
            lines: Vec::new(),
            constants: ValueArray::new(),
        }
    }

    pub fn write(&mut self, byte: u8, line: i32) {
        if self.code.capacity() < self.code.len() + 1 {
            let old_capacity = self.code.capacity();
            let new_capacity = if old_capacity < 8 { 8 } else { old_capacity * 2 };
            self.code.reserve(new_capacity - self.code.len());
        }

        if self.lines.capacity() < self.lines.len() + 1 {
            let old_capacity = self.lines.capacity();
            let new_capacity = if old_capacity < 8 { 8 } else { old_capacity * 2 };
            self.lines.reserve(new_capacity - self.lines.len());
        }

        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn constants(&self) -> &ValueArray {
        &self.constants
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.write(value);
        self.constants.values().len() - 1
    }
}