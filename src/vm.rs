use crate::chunk::{Chunk, OpCode};
use crate::object::Object;
use crate::value::Value;

use std::collections::HashMap;
use std::rc::Rc;

pub struct VM {
    chunk: Option<Chunk>,
    ip: usize,
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
}


macro_rules! runtime_error {
    ($self: ident, $format: literal) => {
        eprintln!($format);

        let line = $self.chunk.as_ref().unwrap().lines()[$self.ip - 1];
        eprintln!("[line {}] in script", line);
        $self.reset_stack();
    };

    ($self: ident, $format: literal, $($arg: tt)*) => {
        eprintln!($format, $($arg)*);

        let line = $self.chunk.as_ref().unwrap().lines()[$self.ip - 1];
        eprintln!("[line {}] in script", line);
        $self.reset_stack();
    }
}

macro_rules! binary_op {
    ($self: ident, $ctor:path, $op: tt) => {
        if let (Value::Number(a), Value::Number(b)) = ($self.peek(0), $self.peek(1)) {
            $self.stack.push($ctor((b $op a)));
        } else {
            runtime_error!($self, "Operands must be numbers.");
            return InterpretResult::RuntimeError;
        }
    }
}

impl VM {
    pub fn new() -> Self {
        Self { chunk: None, ip: 0, stack: Vec::new(), globals: HashMap::new() }
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
                    print!("{:?}", slot);
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
                        chunk.constants().values()[idx as usize].clone()
                    };
                    self.stack.push(constant);
                }
                OpCode::Nil => {
                    self.stack.push(Value::Nil);
                }
                OpCode::True => {
                    self.stack.push(Value::Bool(true));
                }
                OpCode::False => {
                    self.stack.push(Value::Bool(false));
                }
                OpCode::Greater => {
                    binary_op!(self, Value::Bool, >);
                }
                OpCode::Less => {
                    binary_op!(self, Value::Bool, <);
                }
                OpCode::Equal => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Bool(a == b));
                }
                OpCode::Add => {
                    if let (Value::Object(a), Value::Object(b)) = (self.peek(0), self.peek(1))
                    && let (Object::String(a), Object::String(b)) = (a.as_ref(), b.as_ref()) {
                        _ = self.stack.pop().unwrap();
                        _ = self.stack.pop().unwrap();
                        self.stack.push(Value::Object(Rc::new(Object::String(b.clone() + &a))));
                    } else if let (Value::Number(a), Value::Number(b)) = (self.peek(0), self.peek(1)) {
                        _ = self.stack.pop().unwrap();
                        _ = self.stack.pop().unwrap();
                        self.stack.push(Value::Number(a + b));
                    } else {
                        runtime_error!(self, "Operands must be numbers or strings.");
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::Subtract => {
                    binary_op!(self, Value::Number, -);
                }
                OpCode::Multiply => {
                    binary_op!(self, Value::Number, *);
                }
                OpCode::Divide => {
                    binary_op!(self, Value::Number, /);
                }
                OpCode::DefineGlobal => {
                    let constant = {
                        let idx = chunk.code()[self.ip];
                        self.ip += 1;
                        chunk.constants().values()[idx as usize].clone()
                    };

                    if let Value::Object(object) = constant 
                    && let Object::String(name) = object.as_ref()
                    {
                        self.globals.insert(name.clone(), self.stack.pop().unwrap());
                    } else {
                        runtime_error!(self, "Global variable must be a string.");
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::SetGlobal => {
                    let constant = {
                        let idx = chunk.code()[self.ip];
                        self.ip += 1;
                        chunk.constants().values()[idx as usize].clone()
                    };
                    
                    if let Value::Object(object) = constant
                    && let Object::String(name) = object.as_ref()
                    {
                        if self.globals.contains_key(name) {
                            self.globals.insert(name.clone(), self.peek(0).clone());
                        } else {
                            runtime_error!(self, "Undefined variable '{}'.", name);
                            return InterpretResult::RuntimeError;
                        }
                    } else {
                        runtime_error!(self, "Global variable must be a string.");
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::GetGlobal => {
                    let constant = {
                        let idx = chunk.code()[self.ip];
                        self.ip += 1;
                        chunk.constants().values()[idx as usize].clone()
                    };

                    if let Value::Object(object)= constant
                    && let Object::String(name) = object.as_ref()
                    {
                        let value = self.globals.get(name);

                        if let Some(value) = value {
                            self.stack.push(value.clone());
                        } else {
                            runtime_error!(self, "Undefined variable '{}'.", name);
                            return InterpretResult::RuntimeError;
                        }
                    } else {
                        runtime_error!(self, "Global variable must be a string.");
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::Negate => {
                    let Value::Number(value) = self.stack.pop().unwrap() else {
                        runtime_error!(self, "Operand must be a number.");
                        return InterpretResult::RuntimeError;
                    };
                    self.stack.push(Value::Number(-value));
                }
                OpCode::Not => {
                    let value = self.stack.pop().unwrap();
                    let value: bool = value.into();
                    self.stack.push(Value::Bool(!value));
                }
                OpCode::Print => {
                    let value = self.stack.pop().unwrap();
                    println!("{}", value);
                }
                OpCode::Pop => {
                    _ = self.stack.pop().unwrap();
                }
                OpCode::Return => {
                    return InterpretResult::Ok
                }
            }
        }
    }

    fn peek(&self, distance: usize) -> Value {
        self.stack[self.stack.len() - distance - 1].clone()
    }

    fn reset_stack(&mut self) {
        self.stack.clear();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}