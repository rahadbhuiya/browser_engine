// Batch 1 — HTML Tokenizer
//
// A simplified but spec-inspired state machine, following the shape of the WHATWG
// HTML5 tokenization states (https://html.spec.whatwg.org/multipage/parsing.html).
// We don't implement every one of the ~80 spec states yet, but the ones here cover
// real-world HTML: tags, attributes, comments, doctype, and text.
//
// SECURITY NOTE: the tokenizer must never panic on malformed input. Every state
// transition has a defined fallback so garbage/attacker-controlled HTML degrades
// gracefully into text/error-recovery instead of crashing the process.

use super::token::Token;

#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Data,
    TagOpen,
    EndTagOpen,
    TagName,
    BeforeAttributeName,
    AttributeName,
    AfterAttributeName,
    BeforeAttributeValue,
    AttributeValueDoubleQuoted,
    AttributeValueSingleQuoted,
    AttributeValueUnquoted,
    AfterAttributeValueQuoted,
    SelfClosingStartTag,
    MarkupDeclarationOpen,
    CommentStart,
    Comment,
    CommentEnd,
    Doctype,
    DoctypeName,
    BogusComment,
}

pub struct Tokenizer {
    input: Vec<char>,
    pos: usize,
    state: State,
    /// tokens produced so far, drained by `next_token`
    emitted: Vec<Token>,

    // scratch buffers for the token currently being built
    current_tag_name: String,
    current_tag_is_end: bool,
    current_self_closing: bool,
    current_attrs: Vec<(String, String)>,
    current_attr_name: String,
    current_attr_value: String,
    current_comment: String,
    current_doctype_name: String,
    text_buffer: String,
}

impl Tokenizer {
    pub fn new(input: &str) -> Self {
        Tokenizer {
            input: input.chars().collect(),
            pos: 0,
            state: State::Data,
            emitted: Vec::new(),
            current_tag_name: String::new(),
            current_tag_is_end: false,
            current_self_closing: false,
            current_attrs: Vec::new(),
            current_attr_name: String::new(),
            current_attr_value: String::new(),
            current_comment: String::new(),
            current_doctype_name: String::new(),
            text_buffer: String::new(),
        }
    }

    /// Run the whole input through the state machine and return all tokens.
    /// Malformed input never panics: unexpected states fall back to treating
    /// characters as text (see `_ =>` arms below).
    pub fn tokenize(mut self) -> Vec<Token> {
        while self.pos < self.input.len() {
            self.step();
        }
        self.flush_text();
        self.emitted.push(Token::Eof);
        self.emitted
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn flush_text(&mut self) {
        if !self.text_buffer.is_empty() {
            let text = std::mem::take(&mut self.text_buffer);
            self.emitted.push(Token::Text(text));
        }
    }

    fn reset_tag_scratch(&mut self) {
        self.current_tag_name.clear();
        self.current_tag_is_end = false;
        self.current_self_closing = false;
        self.current_attrs.clear();
    }

    fn emit_tag(&mut self) {
        if self.current_tag_is_end {
            self.emitted.push(Token::EndTag {
                name: self.current_tag_name.clone(),
            });
        } else {
            self.emitted.push(Token::StartTag {
                name: self.current_tag_name.clone(),
                attributes: self.current_attrs.clone(),
                self_closing: self.current_self_closing,
            });
        }
        self.reset_tag_scratch();
    }

    fn finish_current_attr(&mut self) {
        if !self.current_attr_name.is_empty() {
            self.current_attrs.push((
                std::mem::take(&mut self.current_attr_name),
                std::mem::take(&mut self.current_attr_value),
            ));
        } else {
            self.current_attr_name.clear();
            self.current_attr_value.clear();
        }
    }

    fn step(&mut self) {
        match self.state {
            State::Data => self.step_data(),
            State::TagOpen => self.step_tag_open(),
            State::EndTagOpen => self.step_end_tag_open(),
            State::TagName => self.step_tag_name(),
            State::BeforeAttributeName => self.step_before_attribute_name(),
            State::AttributeName => self.step_attribute_name(),
            State::AfterAttributeName => self.step_after_attribute_name(),
            State::BeforeAttributeValue => self.step_before_attribute_value(),
            State::AttributeValueDoubleQuoted => self.step_attr_value_quoted('"'),
            State::AttributeValueSingleQuoted => self.step_attr_value_quoted('\''),
            State::AttributeValueUnquoted => self.step_attr_value_unquoted(),
            State::AfterAttributeValueQuoted => self.step_after_attribute_value_quoted(),
            State::SelfClosingStartTag => self.step_self_closing_start_tag(),
            State::MarkupDeclarationOpen => self.step_markup_declaration_open(),
            State::CommentStart => self.step_comment_start(),
            State::Comment => self.step_comment(),
            State::CommentEnd => self.step_comment_end(),
            State::Doctype => self.step_doctype(),
            State::DoctypeName => self.step_doctype_name(),
            State::BogusComment => self.step_bogus_comment(),
        }
    }

    fn step_data(&mut self) {
        match self.advance() {
            Some('<') => {
                self.flush_text();
                self.state = State::TagOpen;
            }
            Some(c) => self.text_buffer.push(c),
            None => {}
        }
    }

    fn step_tag_open(&mut self) {
        match self.peek() {
            Some('/') => {
                self.pos += 1;
                self.state = State::EndTagOpen;
            }
            Some('!') => {
                self.pos += 1;
                self.state = State::MarkupDeclarationOpen;
            }
            Some(c) if c.is_ascii_alphabetic() => {
                self.reset_tag_scratch();
                self.current_tag_is_end = false;
                self.state = State::TagName;
            }
            // Malformed: '<' not followed by anything sensible -> treat as text
            _ => {
                self.text_buffer.push('<');
                self.state = State::Data;
            }
        }
    }

    fn step_end_tag_open(&mut self) {
        match self.peek() {
            Some(c) if c.is_ascii_alphabetic() => {
                self.reset_tag_scratch();
                self.current_tag_is_end = true;
                self.state = State::TagName;
            }
            Some('>') => {
                // "</>" — bogus, just skip it per spec's error-recovery spirit
                self.pos += 1;
                self.state = State::Data;
            }
            _ => {
                // malformed end tag; recover into bogus comment rather than crash
                self.state = State::BogusComment;
            }
        }
    }

    fn step_tag_name(&mut self) {
        match self.advance() {
            Some(c) if c.is_whitespace() => self.state = State::BeforeAttributeName,
            Some('/') => self.state = State::SelfClosingStartTag,
            Some('>') => {
                self.emit_tag();
                self.state = State::Data;
            }
            Some(c) => self.current_tag_name.push(c.to_ascii_lowercase()),
            None => {} // EOF mid-tag: just stop, don't emit malformed tag
        }
    }

    fn step_before_attribute_name(&mut self) {
        match self.peek() {
            Some(c) if c.is_whitespace() => {
                self.pos += 1;
            }
            Some('/') => {
                self.pos += 1;
                self.state = State::SelfClosingStartTag;
            }
            Some('>') => {
                self.pos += 1;
                self.emit_tag();
                self.state = State::Data;
            }
            Some(_) => {
                self.current_attr_name.clear();
                self.current_attr_value.clear();
                self.state = State::AttributeName;
            }
            None => {}
        }
    }

    fn step_attribute_name(&mut self) {
        match self.peek() {
            Some(c) if c.is_whitespace() => {
                self.pos += 1;
                self.state = State::AfterAttributeName;
            }
            Some('=') => {
                self.pos += 1;
                self.state = State::BeforeAttributeValue;
            }
            Some('/') => {
                self.finish_current_attr();
                self.pos += 1;
                self.state = State::SelfClosingStartTag;
            }
            Some('>') => {
                self.finish_current_attr();
                self.pos += 1;
                self.emit_tag();
                self.state = State::Data;
            }
            Some(c) => {
                self.current_attr_name.push(c.to_ascii_lowercase());
                self.pos += 1;
            }
            None => {}
        }
    }

    fn step_after_attribute_name(&mut self) {
        match self.peek() {
            Some(c) if c.is_whitespace() => {
                self.pos += 1;
            }
            Some('=') => {
                self.pos += 1;
                self.state = State::BeforeAttributeValue;
            }
            Some('/') => {
                self.finish_current_attr();
                self.pos += 1;
                self.state = State::SelfClosingStartTag;
            }
            Some('>') => {
                self.finish_current_attr();
                self.pos += 1;
                self.emit_tag();
                self.state = State::Data;
            }
            Some(_) => {
                self.finish_current_attr();
                self.state = State::AttributeName;
            }
            None => {}
        }
    }

    fn step_before_attribute_value(&mut self) {
        match self.peek() {
            Some(c) if c.is_whitespace() => {
                self.pos += 1;
            }
            Some('"') => {
                self.pos += 1;
                self.state = State::AttributeValueDoubleQuoted;
            }
            Some('\'') => {
                self.pos += 1;
                self.state = State::AttributeValueSingleQuoted;
            }
            Some('>') => {
                // "attr=>" malformed; recover by finishing attr with empty value
                self.finish_current_attr();
                self.pos += 1;
                self.emit_tag();
                self.state = State::Data;
            }
            Some(_) => {
                self.state = State::AttributeValueUnquoted;
            }
            None => {}
        }
    }

    fn step_attr_value_quoted(&mut self, quote: char) {
        match self.advance() {
            Some(c) if c == quote => {
                self.finish_current_attr();
                self.state = State::AfterAttributeValueQuoted;
            }
            Some(c) => self.current_attr_value.push(c),
            None => {} // unterminated quoted value at EOF — just stop
        }
    }

    fn step_attr_value_unquoted(&mut self) {
        match self.peek() {
            Some(c) if c.is_whitespace() => {
                self.pos += 1;
                self.finish_current_attr();
                self.state = State::BeforeAttributeName;
            }
            Some('>') => {
                self.finish_current_attr();
                self.pos += 1;
                self.emit_tag();
                self.state = State::Data;
            }
            Some(c) => {
                self.current_attr_value.push(c);
                self.pos += 1;
            }
            None => {}
        }
    }

    fn step_after_attribute_value_quoted(&mut self) {
        match self.peek() {
            Some(c) if c.is_whitespace() => {
                self.pos += 1;
                self.state = State::BeforeAttributeName;
            }
            Some('/') => {
                self.pos += 1;
                self.state = State::SelfClosingStartTag;
            }
            Some('>') => {
                self.pos += 1;
                self.emit_tag();
                self.state = State::Data;
            }
            _ => {
                // malformed spacing; recover by going back to attribute hunting
                self.state = State::BeforeAttributeName;
            }
        }
    }

    fn step_self_closing_start_tag(&mut self) {
        match self.peek() {
            Some('>') => {
                self.pos += 1;
                self.current_self_closing = true;
                self.emit_tag();
                self.state = State::Data;
            }
            _ => {
                // malformed "/" not followed by ">" — recover into attribute hunt
                self.state = State::BeforeAttributeName;
            }
        }
    }

    fn step_markup_declaration_open(&mut self) {
        let rest: String = self.input[self.pos..].iter().collect();
        if rest.starts_with("--") {
            self.pos += 2;
            self.current_comment.clear();
            self.state = State::CommentStart;
        } else if rest.to_ascii_uppercase().starts_with("DOCTYPE") {
            self.pos += "DOCTYPE".len();
            self.current_doctype_name.clear();
            self.state = State::Doctype;
        } else {
            // unknown declaration (e.g. CDATA outside foreign content) -> bogus comment
            self.current_comment.clear();
            self.state = State::BogusComment;
        }
    }

    fn step_comment_start(&mut self) {
        self.state = State::Comment;
    }

    fn step_comment(&mut self) {
        let rest: String = self.input[self.pos..].iter().collect();
        if rest.starts_with("-->") {
            self.pos += 3;
            self.emitted
                .push(Token::Comment(std::mem::take(&mut self.current_comment)));
            self.state = State::Data;
        } else {
            match self.advance() {
                Some(c) => self.current_comment.push(c),
                None => {
                    // unterminated comment at EOF: emit what we have
                    self.emitted
                        .push(Token::Comment(std::mem::take(&mut self.current_comment)));
                }
            }
        }
    }

    fn step_comment_end(&mut self) {
        self.state = State::Data;
    }

    fn step_doctype(&mut self) {
        match self.peek() {
            Some(c) if c.is_whitespace() => self.pos += 1,
            Some('>') => {
                self.pos += 1;
                self.emitted.push(Token::Doctype { name: None });
                self.state = State::Data;
            }
            Some(_) => self.state = State::DoctypeName,
            None => {}
        }
    }

    fn step_doctype_name(&mut self) {
        match self.peek() {
            Some('>') => {
                self.pos += 1;
                let name = if self.current_doctype_name.is_empty() {
                    None
                } else {
                    Some(std::mem::take(&mut self.current_doctype_name))
                };
                self.emitted.push(Token::Doctype { name });
                self.state = State::Data;
            }
            Some(c) => {
                self.current_doctype_name.push(c);
                self.pos += 1;
            }
            None => {}
        }
    }

    fn step_bogus_comment(&mut self) {
        match self.advance() {
            Some('>') => {
                self.emitted
                    .push(Token::Comment(std::mem::take(&mut self.current_comment)));
                self.state = State::Data;
            }
            Some(c) => self.current_comment.push(c),
            None => {
                self.emitted
                    .push(Token::Comment(std::mem::take(&mut self.current_comment)));
            }
        }
    }
}