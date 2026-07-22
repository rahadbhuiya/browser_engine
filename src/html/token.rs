// Batch 1 — HTML token types
// These mirror the token categories described in the WHATWG HTML5 tokenization spec:
// https://html.spec.whatwg.org/multipage/parsing.html#tokenization

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Doctype {
        name: Option<String>,
    },
    StartTag {
        name: String,
        attributes: Vec<(String, String)>,
        self_closing: bool,
    },
    EndTag {
        name: String,
    },
    Comment(String),
    Text(String),
    Eof,
}