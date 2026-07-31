use super::ast::{BinaryOp, Expr, Literal, Program, Stmt};
use super::lexer::Token;

#[derive(PartialEq, PartialOrd, Clone, Copy)]
enum Precedence {
    None,
    Assignment, // =
    Equality,   // == != ===
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Call,       // . ()
}

impl Precedence {
    fn from_token(token: &Token) -> Precedence {
        match token {
            Token::Equal => Precedence::Assignment,
            Token::EqualEqual | Token::EqualEqualEqual | Token::NotEqual => Precedence::Equality,
            Token::Less | Token::LessEqual | Token::Greater | Token::GreaterEqual => {
                Precedence::Comparison
            }
            Token::Plus | Token::Minus => Precedence::Term,
            Token::Star | Token::Slash => Precedence::Factor,
            Token::Dot | Token::LParen => Precedence::Call,
            _ => Precedence::None,
        }
    }
}

pub struct JsParser {
    tokens: Vec<Token>,
    pos: usize,
}

impl JsParser {
    pub fn new(tokens: Vec<Token>) -> Self {
        JsParser { tokens, pos: 0 }
    }

    pub fn parse(mut self) -> Program {
        let mut body = Vec::new();
        while !self.is_at_end() {
            if let Some(stmt) = self.parse_statement() {
                body.push(stmt);
            } else {
                // Advance to avoid infinite loop on unexpected token
                self.advance();
            }
        }
        Program { body }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let t = self.peek().clone();
        if !self.is_at_end() {
            self.pos += 1;
        }
        t
    }

    fn check(&self, expected: &Token) -> bool {
        std::mem::discriminant(self.peek()) == std::mem::discriminant(expected)
    }

    fn match_token(&mut self, expected: &Token) -> bool {
        if self.check(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume_semicolon(&mut self) {
        let _ = self.match_token(&Token::Semicolon);
    }

    fn parse_statement(&mut self) -> Option<Stmt> {
        match self.peek() {
            Token::Keyword(kw) => match kw.as_str() {
                "var" | "let" | "const" => self.parse_var_declaration(),
                "if" => self.parse_if_statement(),
                "while" => self.parse_while_statement(),
                "function" => self.parse_function_declaration(),
                "return" => self.parse_return_statement(),
                _ => self.parse_expr_statement(),
            },
            Token::LBrace => self.parse_block_statement(),
            _ => self.parse_expr_statement(),
        }
    }

    fn parse_var_declaration(&mut self) -> Option<Stmt> {
        self.advance(); // consume var/let/const
        let name = match self.advance() {
            Token::Ident(n) => n,
            _ => return None,
        };

        let initializer = if self.match_token(&Token::Equal) {
            self.parse_expression(Precedence::None)
        } else {
            None
        };

        self.consume_semicolon();
        Some(Stmt::VarDecl { name, initializer })
    }

    fn parse_if_statement(&mut self) -> Option<Stmt> {
        self.advance(); // consume 'if'
        if !self.match_token(&Token::LParen) {
            return None;
        }
        let condition = self.parse_expression(Precedence::None)?;
        if !self.match_token(&Token::RParen) {
            return None;
        }

        let then_branch = Box::new(self.parse_statement()?);
        let else_branch = if matches!(self.peek(), Token::Keyword(kw) if kw == "else") {
            self.advance(); // consume 'else'
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };

        Some(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn parse_while_statement(&mut self) -> Option<Stmt> {
        self.advance(); // consume 'while'
        if !self.match_token(&Token::LParen) {
            return None;
        }
        let condition = self.parse_expression(Precedence::None)?;
        if !self.match_token(&Token::RParen) {
            return None;
        }
        let body = Box::new(self.parse_statement()?);
        Some(Stmt::While { condition, body })
    }

    fn parse_function_declaration(&mut self) -> Option<Stmt> {
        self.advance(); // consume 'function'
        let name = match self.advance() {
            Token::Ident(n) => n,
            _ => return None,
        };

        if !self.match_token(&Token::LParen) {
            return None;
        }
        let mut params = Vec::new();
        if !self.check(&Token::RParen) {
            loop {
                if let Token::Ident(p) = self.advance() {
                    params.push(p);
                }
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        if !self.match_token(&Token::RParen) {
            return None;
        }

        let body = match self.parse_block_statement()? {
            Stmt::Block(stmts) => stmts,
            _ => return None,
        };

        Some(Stmt::FunctionDecl { name, params, body })
    }

    fn parse_return_statement(&mut self) -> Option<Stmt> {
        self.advance(); // consume 'return'
        let expr = if self.check(&Token::Semicolon) || self.check(&Token::RBrace) || self.is_at_end() {
            None
        } else {
            self.parse_expression(Precedence::None)
        };
        self.consume_semicolon();
        Some(Stmt::Return(expr))
    }

    fn parse_block_statement(&mut self) -> Option<Stmt> {
        self.advance(); // consume '{'
        let mut stmts = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            if let Some(s) = self.parse_statement() {
                stmts.push(s);
            } else {
                self.advance();
            }
        }
        self.match_token(&Token::RBrace);
        Some(Stmt::Block(stmts))
    }

    fn parse_expr_statement(&mut self) -> Option<Stmt> {
        let expr = self.parse_expression(Precedence::None)?;
        self.consume_semicolon();
        Some(Stmt::Expr(expr))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expr> {
        let mut left = self.parse_prefix()?;

        while !self.is_at_end() {
            let next_prec = Precedence::from_token(self.peek());
            if precedence >= next_prec {
                break;
            }
            if let Some(new_left) = self.parse_infix(left.clone(), next_prec) {
                left = new_left;
            } else {
                break;
            }
        }

        Some(left)
    }

    fn parse_prefix(&mut self) -> Option<Expr> {
        let tok = self.advance();
        match tok {
            Token::Number(n) => Some(Expr::Literal(Literal::Number(n))),
            Token::String(s) => Some(Expr::Literal(Literal::String(s))),
            Token::Keyword(kw) => match kw.as_str() {
                "true" => Some(Expr::Literal(Literal::Boolean(true))),
                "false" => Some(Expr::Literal(Literal::Boolean(false))),
                "null" => Some(Expr::Literal(Literal::Null)),
                "undefined" => Some(Expr::Literal(Literal::Undefined)),
                _ => None,
            },
            Token::Ident(name) => Some(Expr::Identifier(name)),
            Token::LParen => {
                let expr = self.parse_expression(Precedence::None)?;
                self.match_token(&Token::RParen);
                Some(expr)
            }
            _ => None,
        }
    }

    fn parse_infix(&mut self, left: Expr, prec: Precedence) -> Option<Expr> {
        let tok = self.advance();
        match tok {
            Token::Equal => {
                let right = self.parse_expression(Precedence::Assignment)?;
                Some(Expr::Assign {
                    target: Box::new(left),
                    value: Box::new(right),
                })
            }
            Token::Plus | Token::Minus | Token::Star | Token::Slash | Token::EqualEqual
            | Token::EqualEqualEqual | Token::NotEqual | Token::Less | Token::LessEqual
            | Token::Greater | Token::GreaterEqual => {
                let op = match tok {
                    Token::Plus => BinaryOp::Add,
                    Token::Minus => BinaryOp::Sub,
                    Token::Star => BinaryOp::Mul,
                    Token::Slash => BinaryOp::Div,
                    Token::EqualEqual => BinaryOp::Equal,
                    Token::EqualEqualEqual => BinaryOp::StrictEqual,
                    Token::NotEqual => BinaryOp::NotEqual,
                    Token::Less => BinaryOp::LessThan,
                    Token::LessEqual => BinaryOp::LessEqual,
                    Token::Greater => BinaryOp::GreaterThan,
                    Token::GreaterEqual => BinaryOp::GreaterEqual,
                    _ => unreachable!(),
                };
                let right = self.parse_expression(prec)?;
                Some(Expr::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
            Token::Dot => {
                if let Token::Ident(prop) = self.advance() {
                    Some(Expr::Member {
                        object: Box::new(left),
                        property: prop,
                    })
                } else {
                    None
                }
            }
            Token::LParen => {
                let mut args = Vec::new();
                if !self.check(&Token::RParen) {
                    loop {
                        if let Some(arg) = self.parse_expression(Precedence::None) {
                            args.push(arg);
                        }
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                }
                self.match_token(&Token::RParen);
                Some(Expr::Call {
                    callee: Box::new(left),
                    args,
                })
            }
            _ => None,
        }
    }
}
