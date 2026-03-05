use crate::object::Object;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
    Object(Rc<Object>),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::Number(n) => write!(f, "{}", n),
            Value::Object(o) => write!(f, "{}", o),
        }
    }
}

impl Into<bool> for Value {
    fn into(self) -> bool {
        match self {
            Value::Bool(b) => b,
            Value::Nil => false,
            _ => true
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => a == b,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValueArray {
    values: Vec<Value>,
}

impl ValueArray {
    pub fn values(&self) -> &[Value] {
        &self.values
    }

    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn write(&mut self, value: Value) {
        if self.values.capacity() < self.values.len() + 1 {
            let old_capacity = self.values.capacity();
            let new_capacity = if old_capacity < 8 { 8 } else { old_capacity * 2 };
            self.values.reserve(new_capacity - self.values.len());
        }
        self.values.push(value);
    }
}