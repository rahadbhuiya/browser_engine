#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Keyword(String),
    Ident(String),
    Number(f32),
    String(String),
    Plus,            // +
    Minus,           // -
    Star,            // *
    Slash,           // /
    Equal,           // =
    EqualEqual,      // ==
    EqualEqualEqual, // ===
    NotEqual,        // !=
    Less,            // <
    LessEqual,       // <=
    Greater,         // >
    GreaterEqual,    // >=
    Dot,             // .
    Comma,           // ,
    Semicolon,       // ;
    LParen,          // (
    RParen,          // )
    LBrace,          // {
    RBrace,          // }
    Eof,
}

pub struct JsLexer<'a> {
    input: &'a str,
    chars: Vec<char>,
    pos: usize,
}

impl<'a> JsLexer<'a> {
    pub fn new(input: &'a str) -> Self {
        JsLexer {
            input,
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    pub fn tokenize(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while self.pos < self.chars.len() {
            self.skip_whitespace_and_comments();
            if self.pos >= self.chars.len() {
                break;
            }

            let ch = self.chars[self.pos];
            if ch.is_ascii_alphabetic() || ch == '_' || ch == '$' {
                tokens.push(self.read_identifier_or_keyword());
            } else if ch.is_ascii_digit() {
                tokens.push(self.read_number());
            } else if ch == '"' || ch == '\'' {
                tokens.push(self.read_string(ch));
            } else {
                if let Some(tok) = self.read_operator_or_punct() {
                    tokens.push(tok);
                } else {
                    // Skip unknown character gracefully
                    self.pos += 1;
                }
            }
        }
        tokens.push(Token::Eof);
        tokens
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn skip_whitespace_and_comments(&mut self) {
        while self.pos < self.chars.len() {
            let ch = self.chars[self.pos];
            if ch.is_whitespace() {
                self.pos += 1;
            } else if ch == '/' && self.pos + 1 < self.chars.len() {
                if self.chars[self.pos + 1] == '/' {
                    // Line comment
                    self.pos += 2;
                    while self.pos < self.chars.len() && self.chars[self.pos] != '\n' {
                        self.pos += 1;
                    }
                } else if self.chars[self.pos + 1] == '*' {
                    // Block comment
                    self.pos += 2;
                    while self.pos + 1 < self.chars.len() {
                        if self.chars[self.pos] == '*' && self.chars[self.pos + 1] == '/' {
                            self.pos += 2;
                            break;
                        }
                        self.pos += 1;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn read_identifier_or_keyword(&mut self) -> Token {
        let start = self.pos;
        while self.pos < self.chars.len() {
            let ch = self.chars[self.pos];
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '$' {
                self.pos += 1;
            } else {
                break;
            }
        }
        let word: String = self.chars[start..self.pos].iter().collect();
        match word.as_str() {
            "var" | "let" | "const" | "function" | "if" | "else" | "while" | "return" | "true"
            | "false" | "null" | "undefined" => Token::Keyword(word),
            _ => Token::Ident(word),
        }
    }

    fn read_number(&mut self) -> Token {
        let start = self.pos;
        let mut has_dot = false;
        while self.pos < self.chars.len() {
            let ch = self.chars[self.pos];
            if ch.is_ascii_digit() {
                self.pos += 1;
            } else if ch == '.' && !has_dot {
                has_dot = true;
                self.pos += 1;
            } else {
                break;
            }
        }
        let num_str: String = self.chars[start..self.pos].iter().collect();
        let val = num_str.parse::<f32>().unwrap_or(0.0);
        Token::Number(val)
    }

    fn read_string(&mut self, quote: char) -> Token {
        self.pos += 1; // consume opening quote
        let mut str_buf = String::new();
        while self.pos < self.chars.len() {
            let ch = self.chars[self.pos];
            if ch == quote {
                self.pos += 1; // consume closing quote
                break;
            } else if ch == '\\' && self.pos + 1 < self.chars.len() {
                self.pos += 1;
                let next_ch = self.chars[self.pos];
                match next_ch {
                    'n' => str_buf.push('\n'),
                    't' => str_buf.push('\t'),
                    'r' => str_buf.push('\r'),
                    _ => str_buf.push(next_ch),
                }
                self.pos += 1;
            } else {
                str_buf.push(ch);
                self.pos += 1;
            }
        }
        Token::String(str_buf)
    }

    fn read_operator_or_punct(&mut self) -> Option<Token> {
        let ch = self.advance()?;
        match ch {
            '+' => Some(Token::Plus),
            '-' => Some(Token::Minus),
            '*' => Some(Token::Star),
            '/' => Some(Token::Slash),
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Some(Token::EqualEqualEqual)
                    } else {
                        Some(Token::EqualEqual)
                    }
                } else {
                    Some(Token::Equal)
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Some(Token::NotEqual)
                } else {
                    None
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Some(Token::LessEqual)
                } else {
                    Some(Token::Less)
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Some(Token::GreaterEqual)
                } else {
                    Some(Token::Greater)
                }
            }
            '.' => Some(Token::Dot),
            ',' => Some(Token::Comma),
            ';' => Some(Token::Semicolon),
            '(' => Some(Token::LParen),
            ')' => Some(Token::RParen),
            '{' => Some(Token::LBrace),
            '}' => Some(Token::RBrace),
            _ => None,
        }
    }
}
