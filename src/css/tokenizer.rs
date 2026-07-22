// Batch 2 — CSS Tokenizer
//
// Scans raw CSS source into a CssToken stream. Not a full implementation of the
// CSS Syntax Module state machine (no escape sequences, url(), unicode-range yet),
// but enough to tokenize real-world stylesheets: selectors, declarations, strings,
// numbers, comments.
//
// SECURITY NOTE: same rule as the HTML tokenizer — malformed/truncated CSS must
// never panic. Every branch has a defined fallback (see `advance_or_stop` usage
// and the `None =>` arms below).

use super::token::CssToken;

pub struct CssTokenizer {
    input: Vec<char>,
    pos: usize,
}

impl CssTokenizer {
    pub fn new(input: &str) -> Self {
        CssTokenizer {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    pub fn tokenize(mut self) -> Vec<CssToken> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            let is_eof = tok == CssToken::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        tokens
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.input.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn skip_comment(&mut self) -> bool {
        if self.peek() == Some('/') && self.peek_at(1) == Some('*') {
            self.pos += 2;
            while self.pos < self.input.len() {
                if self.peek() == Some('*') && self.peek_at(1) == Some('/') {
                    self.pos += 2;
                    break;
                }
                self.pos += 1;
            }
            true
        } else {
            false
        }
    }

    fn is_ident_start(c: char) -> bool {
        c.is_alphabetic() || c == '_' || c == '-' || !c.is_ascii()
    }

    fn is_ident_char(c: char) -> bool {
        c.is_alphanumeric() || c == '_' || c == '-' || !c.is_ascii()
    }

    fn consume_ident(&mut self) -> String {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if Self::is_ident_char(c) {
                s.push(c);
                self.pos += 1;
            } else {
                break;
            }
        }
        s
    }

    fn consume_number(&mut self) -> f64 {
        let start = self.pos;
        if self.peek() == Some('-') || self.peek() == Some('+') {
            self.pos += 1;
        }
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.pos += 1;
        }
        if self.peek() == Some('.') && matches!(self.peek_at(1), Some(c) if c.is_ascii_digit()) {
            self.pos += 1;
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        let s: String = self.input[start..self.pos].iter().collect();
        s.parse::<f64>().unwrap_or(0.0)
    }

    fn consume_string(&mut self, quote: char) -> String {
        self.pos += 1; // opening quote
        let mut s = String::new();
        loop {
            match self.peek() {
                Some(c) if c == quote => {
                    self.pos += 1;
                    break;
                }
                Some(c) => {
                    s.push(c);
                    self.pos += 1;
                }
                None => break, // unterminated string at EOF: stop, don't panic
            }
        }
        s
    }

    fn next_token(&mut self) -> CssToken {
        // comments can appear anywhere; skip zero or more before real tokenizing
        while self.skip_comment() {}

        let c = match self.peek() {
            Some(c) => c,
            None => return CssToken::Eof,
        };

        if c.is_whitespace() {
            while matches!(self.peek(), Some(c) if c.is_whitespace()) {
                self.pos += 1;
            }
            return CssToken::Whitespace;
        }

        match c {
            '{' => {
                self.pos += 1;
                CssToken::LeftBrace
            }
            '}' => {
                self.pos += 1;
                CssToken::RightBrace
            }
            '(' => {
                self.pos += 1;
                CssToken::LeftParen
            }
            ')' => {
                self.pos += 1;
                CssToken::RightParen
            }
            '[' => {
                self.pos += 1;
                CssToken::LeftBracket
            }
            ']' => {
                self.pos += 1;
                CssToken::RightBracket
            }
            ':' => {
                self.pos += 1;
                CssToken::Colon
            }
            ';' => {
                self.pos += 1;
                CssToken::Semicolon
            }
            ',' => {
                self.pos += 1;
                CssToken::Comma
            }
            '"' | '\'' => CssToken::Str(self.consume_string(c)),
            '#' => {
                self.pos += 1;
                let name = self.consume_ident();
                if name.is_empty() {
                    CssToken::Delim('#')
                } else {
                    CssToken::Hash(name)
                }
            }
            '@' => {
                self.pos += 1;
                let name = self.consume_ident();
                CssToken::AtKeyword(name)
            }
            c if c.is_ascii_digit() => self.number_or_percentage(),
            '-' if matches!(self.peek_at(1), Some(c2) if c2.is_ascii_digit()) => {
                self.number_or_percentage()
            }
            c if Self::is_ident_start(c) => CssToken::Ident(self.consume_ident()),
            '%' => {
                self.pos += 1;
                CssToken::Delim('%')
            }
            other => {
                self.pos += 1;
                CssToken::Delim(other)
            }
        }
    }

    fn number_or_percentage(&mut self) -> CssToken {
        let n = self.consume_number();
        if self.peek() == Some('%') {
            self.pos += 1;
            CssToken::Percentage(n)
        } else {
            CssToken::Number(n)
        }
    }
}