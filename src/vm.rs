use crate::chunk::{Chunk, OpCode};
use crate::value::Value;

pub struct VM {
    chunk: Option<Chunk>,
    ip: usize,
    stack: Vec<Value>,
}

macro_rules! binary_op {
    ($self: ident, $op: tt) => {
        let a = $self.stack.pop().unwrap();
        let b = $self.stack.pop().unwrap();
        $self.stack.push(a $op b);
    }
}

impl VM {
    pub fn new() -> Self {
        Self { chunk: None, ip: 0, stack: Vec::new() }
    }

    pub fn interpret(&mut self, source: &str) -> InterpretResult {
        let mut chunk = Chunk::new();

        if !crate::compiler::compile(source, &mut chunk) {
            return InterpretResult::CompileError;
        }

        self.chunk = Some(chunk);
        self.ip = 0;

        return self.run();
    }

    fn run(&mut self) -> InterpretResult {
        let chunk = self.chunk.as_ref().unwrap();
        loop {
            #[cfg(feature = "debug_trace_execution")]
            {
                print!("          ");
                for slot in self.stack.iter() {
                    print!("[ ");
                    print!("{}", slot);
                    print!(" ]");
                }
                println!();
                chunk.disassemble_instruction(self.ip);
            }
            
            let instruction = OpCode::from(chunk.code()[self.ip]);
            self.ip += 1;

            match instruction {
                OpCode::Constant => {
                    let constant = {
                        let idx = chunk.code()[self.ip];
                        self.ip += 1;
                        chunk.constants().values()[idx as usize]
                    };
                    self.stack.push(constant);
                }
                OpCode::Add => {
                    binary_op!(self, +);
                }
                OpCode::Subtract => {
                    binary_op!(self, -);
                }
                OpCode::Multiply => {
                    binary_op!(self, *);
                }
                OpCode::Divide => {
                    binary_op!(self, /);
                }
                OpCode::Negate => {
                    let value = self.stack.pop().unwrap();
                    self.stack.push(-value);
                }
                OpCode::Return => {
                    let value = self.stack.pop().unwrap();
                    println!("{}", value);
                    return InterpretResult::Ok
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}