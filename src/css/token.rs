//! CSS token definitions per CSS Syntax Module Level 3.

#[derive(Debug, Clone, PartialEq)]
pub enum CssToken {
    Ident(String),
    AtKeyword(String),
    Hash(String),
    Str(String),
    Number(f64),
    Percentage(f64),
    Delim(char),
    Whitespace,
    Colon,
    Semicolon,
    Comma,
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Eof,
}
