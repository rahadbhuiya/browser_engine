// Batch 2 — CSS token types
// Loosely follows the token categories from the CSS Syntax Module Level 3 spec:
// https://www.w3.org/TR/css-syntax-3/#tokenization
// (simplified: we don't yet handle unicode-range, url(), or escaped characters)

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