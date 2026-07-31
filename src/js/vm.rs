use std::collections::HashMap;
use std::sync::Arc;
use super::bytecode::{Chunk, OpCode, Value};
use super::gc::GarbageCollector;


pub type NativeFnHandler = Arc<dyn Fn(&mut VM, &[Value]) -> Value>;

pub struct VM {
    pub stack: Vec<Value>,
    pub globals: HashMap<String, Value>,
    pub native_fns: HashMap<String, NativeFnHandler>,
    pub gc: GarbageCollector,
}

impl VM {
    pub fn new() -> Self {
        let mut vm = VM {
            stack: Vec::new(),
            globals: HashMap::new(),
            native_fns: HashMap::new(),
            gc: GarbageCollector::new(),
        };
        vm.register_builtins();
        vm
    }

    fn register_builtins(&mut self) {
        self.register_native_fn("log", |_vm, args| {
            let output: Vec<String> = args.iter().map(|a| a.to_string()).collect();
            println!("[JS Console] {}", output.join(" "));
            Value::Undefined
        });
    }

    pub fn register_native_fn<F>(&mut self, name: &str, handler: F)
    where
        F: Fn(&mut VM, &[Value]) -> Value + 'static,
    {
        self.globals.insert(name.to_string(), Value::NativeFn(name.to_string()));
        self.native_fns.insert(name.to_string(), Arc::new(handler));
    }



    pub fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    pub fn pop(&mut self) -> Value {
        self.stack.pop().unwrap_or(Value::Undefined)
    }

    pub fn run(&mut self, chunk: &Chunk) -> Value {
        let mut ip = 0;
        while ip < chunk.code.len() {
            let op = &chunk.code[ip];
            ip += 1;

            match op {
                OpCode::PushConst(val) => {
                    self.push(val.clone());
                }
                OpCode::Pop => {
                    self.pop();
                }
                OpCode::GetVar(name) => {
                    let val = self.globals.get(name).cloned().unwrap_or(Value::Undefined);
                    self.push(val);
                }
                OpCode::SetVar(name) => {
                    let val = self.pop();
                    self.globals.insert(name.clone(), val);
                }
                OpCode::GetMember(prop) => {
                    let obj_val = self.pop();
                    let res = match obj_val {
                        Value::String(s) if prop == "length" => Value::Number(s.len() as f32),
                        _ => Value::Undefined,
                    };
                    self.push(res);
                }
                OpCode::SetMember(_prop) => {
                    let _obj = self.pop();
                    let _val = self.pop();
                }
                OpCode::Add => {
                    let b = self.pop();
                    let a = self.pop();
                    match (a, b) {
                        (Value::Number(n1), Value::Number(n2)) => self.push(Value::Number(n1 + n2)),
                        (Value::String(s1), Value::String(s2)) => self.push(Value::String(format!("{}{}", s1, s2))),
                        (Value::String(s1), Value::Number(n2)) => self.push(Value::String(format!("{}{}", s1, n2))),
                        (Value::Number(n1), Value::String(s2)) => self.push(Value::String(format!("{}{}", n1, s2))),
                        _ => self.push(Value::Undefined),
                    }
                }
                OpCode::Sub => {
                    let b = self.pop();
                    let a = self.pop();
                    match (a, b) {
                        (Value::Number(n1), Value::Number(n2)) => self.push(Value::Number(n1 - n2)),
                        _ => self.push(Value::Undefined),
                    }
                }
                OpCode::Mul => {
                    let b = self.pop();
                    let a = self.pop();
                    match (a, b) {
                        (Value::Number(n1), Value::Number(n2)) => self.push(Value::Number(n1 * n2)),
                        _ => self.push(Value::Undefined),
                    }
                }
                OpCode::Div => {
                    let b = self.pop();
                    let a = self.pop();
                    match (a, b) {
                        (Value::Number(n1), Value::Number(n2)) => self.push(Value::Number(n1 / n2)),
                        _ => self.push(Value::Undefined),
                    }
                }
                OpCode::Equal => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Boolean(a == b));
                }
                OpCode::LessThan => {
                    let b = self.pop();
                    let a = self.pop();
                    match (a, b) {
                        (Value::Number(n1), Value::Number(n2)) => self.push(Value::Boolean(n1 < n2)),
                        _ => self.push(Value::Boolean(false)),
                    }
                }
                OpCode::GreaterThan => {
                    let b = self.pop();
                    let a = self.pop();
                    match (a, b) {
                        (Value::Number(n1), Value::Number(n2)) => self.push(Value::Boolean(n1 > n2)),
                        _ => self.push(Value::Boolean(false)),
                    }
                }
                OpCode::Jump(target) => {
                    ip = *target;
                }
                OpCode::JumpIfFalse(target) => {
                    let cond = self.pop();
                    let is_truthy = match cond {
                        Value::Boolean(b) => b,
                        Value::Number(n) => n != 0.0,
                        Value::String(s) => !s.is_empty(),
                        Value::Null | Value::Undefined => false,
                        _ => true,
                    };
                    if !is_truthy {
                        ip = *target;
                    }
                }
                OpCode::Call(arg_count) => {
                    let callee = self.pop();
                    let mut args = Vec::new();
                    for _ in 0..*arg_count {
                        args.push(self.pop());
                    }
                    args.reverse();

                    if let Value::NativeFn(name) = callee {
                        if let Some(handler) = self.native_fns.get(&name).cloned() {
                            let res = handler(self, &args);
                            self.push(res);
                        } else {
                            self.push(Value::Undefined);
                        }
                    } else {
                        self.push(Value::Undefined);
                    }
                }
                OpCode::Return => {
                    let val = self.pop();
                    return val;
                }
            }
        }
        self.stack.last().cloned().unwrap_or(Value::Undefined)
    }
}
