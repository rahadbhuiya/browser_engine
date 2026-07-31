use super::ast::{BinaryOp, Expr, Literal, Program, Stmt};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Undefined,
    Null,
    Boolean(bool),
    Number(f32),
    String(String),
    NativeFn(String),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Undefined => write!(f, "undefined"),
            Value::Null => write!(f, "null"),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::NativeFn(name) => write!(f, "[native function {}]", name),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OpCode {
    PushConst(Value),
    Pop,
    GetVar(String),
    SetVar(String),
    GetMember(String),
    SetMember(String),
    Add,
    Sub,
    Mul,
    Div,
    Equal,
    LessThan,
    GreaterThan,
    Jump(usize),
    JumpIfFalse(usize),
    Call(usize), // arg_count
    Return,
}

#[derive(Debug, Clone, Default)]
pub struct Chunk {
    pub code: Vec<OpCode>,
}

impl Chunk {
    pub fn emit(&mut self, op: OpCode) -> usize {
        self.code.push(op);
        self.code.len() - 1
    }
}

pub struct Compiler {
    chunk: Chunk,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            chunk: Chunk::default(),
        }
    }

    pub fn compile(mut self, program: &Program) -> Chunk {
        for stmt in &program.body {
            self.compile_stmt(stmt);
        }
        self.chunk
    }

    fn compile_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl { name, initializer } => {
                if let Some(init) = initializer {
                    self.compile_expr(init);
                } else {
                    self.chunk.emit(OpCode::PushConst(Value::Undefined));
                }
                self.chunk.emit(OpCode::SetVar(name.clone()));
            }
            Stmt::Expr(expr) => {
                self.compile_expr(expr);
                self.chunk.emit(OpCode::Pop);
            }
            Stmt::Block(stmts) => {
                for s in stmts {
                    self.compile_stmt(s);
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.compile_expr(condition);
                let jump_false_idx = self.chunk.emit(OpCode::JumpIfFalse(0));
                self.compile_stmt(then_branch);

                if let Some(else_stmt) = else_branch {
                    let jump_end_idx = self.chunk.emit(OpCode::Jump(0));
                    let else_start = self.chunk.code.len();
                    self.chunk.code[jump_false_idx] = OpCode::JumpIfFalse(else_start);
                    self.compile_stmt(else_stmt);
                    let end = self.chunk.code.len();
                    self.chunk.code[jump_end_idx] = OpCode::Jump(end);
                } else {
                    let else_start = self.chunk.code.len();
                    self.chunk.code[jump_false_idx] = OpCode::JumpIfFalse(else_start);
                }
            }
            Stmt::While { condition, body } => {
                let loop_start = self.chunk.code.len();
                self.compile_expr(condition);
                let jump_false_idx = self.chunk.emit(OpCode::JumpIfFalse(0));
                self.compile_stmt(body);
                self.chunk.emit(OpCode::Jump(loop_start));
                let loop_end = self.chunk.code.len();
                self.chunk.code[jump_false_idx] = OpCode::JumpIfFalse(loop_end);
            }
            Stmt::FunctionDecl { .. } => {
                // Function declarations can be extended for user functions
            }
            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    self.compile_expr(e);
                } else {
                    self.chunk.emit(OpCode::PushConst(Value::Undefined));
                }
                self.chunk.emit(OpCode::Return);
            }
        }
    }

    fn compile_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(lit) => {
                let val = match lit {
                    Literal::Number(n) => Value::Number(*n),
                    Literal::String(s) => Value::String(s.clone()),
                    Literal::Boolean(b) => Value::Boolean(*b),
                    Literal::Null => Value::Null,
                    Literal::Undefined => Value::Undefined,
                };
                self.chunk.emit(OpCode::PushConst(val));
            }
            Expr::Identifier(name) => {
                self.chunk.emit(OpCode::GetVar(name.clone()));
            }
            Expr::Binary { op, left, right } => {
                self.compile_expr(left);
                self.compile_expr(right);
                match op {
                    BinaryOp::Add => self.chunk.emit(OpCode::Add),
                    BinaryOp::Sub => self.chunk.emit(OpCode::Sub),
                    BinaryOp::Mul => self.chunk.emit(OpCode::Mul),
                    BinaryOp::Div => self.chunk.emit(OpCode::Div),
                    BinaryOp::Equal | BinaryOp::StrictEqual | BinaryOp::NotEqual => {
                        self.chunk.emit(OpCode::Equal)
                    }
                    BinaryOp::LessThan => self.chunk.emit(OpCode::LessThan),
                    BinaryOp::GreaterThan => self.chunk.emit(OpCode::GreaterThan),
                    _ => self.chunk.emit(OpCode::Equal),
                };
            }
            Expr::Assign { target, value } => {
                self.compile_expr(value);
                match &**target {
                    Expr::Identifier(name) => {
                        self.chunk.emit(OpCode::SetVar(name.clone()));
                    }
                    Expr::Member { object, property } => {
                        self.compile_expr(object);
                        self.chunk.emit(OpCode::SetMember(property.clone()));
                    }
                    _ => {}
                }
            }
            Expr::Member { object, property } => {
                self.compile_expr(object);
                self.chunk.emit(OpCode::GetMember(property.clone()));
            }
            Expr::Call { callee, args } => {
                for arg in args {
                    self.compile_expr(arg);
                }
                self.compile_expr(callee);
                self.chunk.emit(OpCode::Call(args.len()));
            }
        }
    }
}
