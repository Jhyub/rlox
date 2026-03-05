#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::Number(n) => write!(f, "{}", n),
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