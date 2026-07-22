// Batch 2 — CSS Parser
//
// Turns a CssToken stream into a Stylesheet (CSSOM). Supports simple selectors
// (type, class, id, universal), the four combinators (descendant, child,
// next-sibling, subsequent-sibling), declaration blocks, and `!important`.
// At-rules (@media, @font-face, ...) are recognized but skipped for now rather
// than causing a parse failure — real stylesheets always contain at-rules, and
// a browser that chokes on the first one it doesn't support is a security and
// robustness problem, not just a missing-feature problem.

use super::cssom::{Combinator, Declaration, Rule, Selector, SimpleSelector, Stylesheet};
use super::token::CssToken;

pub struct CssParser {
    tokens: Vec<CssToken>,
    pos: usize,
}

impl CssParser {
    pub fn new(tokens: Vec<CssToken>) -> Self {
        CssParser { tokens, pos: 0 }
    }

    pub fn parse(mut self) -> Stylesheet {
        let mut rules = Vec::new();
        loop {
            self.skip_whitespace();
            match self.peek() {
                None | Some(CssToken::Eof) => break,
                Some(CssToken::AtKeyword(_)) => self.skip_at_rule(),
                Some(CssToken::RightBrace) => {
                    // stray closing brace from malformed input — skip and continue
                    self.pos += 1;
                }
                _ => {
                    let pos_before = self.pos;
                    if let Some(rule) = self.parse_rule() {
                        rules.push(rule);
                    }
                    // Guarantee forward progress: if parsing a rule consumed
                    // nothing (e.g. malformed selector with no block), force
                    // the cursor forward by one token so we never loop forever
                    // on malformed input.
                    if self.pos == pos_before {
                        self.pos += 1;
                    }
                }
            }
        }
        Stylesheet { rules }
    }

    fn peek(&self) -> Option<&CssToken> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<CssToken> {
        let t = self.tokens.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(CssToken::Whitespace)) {
            self.pos += 1;
        }
    }

    /// Skip an at-rule entirely: either `@foo ...;` or `@foo ... { ... }`.
    /// Never panics even if the block is unterminated (stops at EOF).
    fn skip_at_rule(&mut self) {
        self.pos += 1; // consume the AtKeyword token
        loop {
            match self.peek() {
                None | Some(CssToken::Eof) => return,
                Some(CssToken::Semicolon) => {
                    self.pos += 1;
                    return;
                }
                Some(CssToken::LeftBrace) => {
                    self.skip_balanced_block();
                    return;
                }
                _ => {
                    self.pos += 1;
                }
            }
        }
    }

    /// Consumes a `{ ... }` block, correctly handling nested braces (needed for
    /// @media blocks which contain full rules with their own braces).
    fn skip_balanced_block(&mut self) {
        let mut depth = 0;
        loop {
            match self.advance() {
                Some(CssToken::LeftBrace) => depth += 1,
                Some(CssToken::RightBrace) => {
                    depth -= 1;
                    if depth == 0 {
                        return;
                    }
                }
                None => return, // unterminated block at EOF
                _ => {}
            }
        }
    }

    fn parse_rule(&mut self) -> Option<Rule> {
        let selectors = self.parse_selector_list()?;
        self.skip_whitespace();
        if !matches!(self.peek(), Some(CssToken::LeftBrace)) {
            // Either malformed input, or selector syntax we don't support yet
            // (e.g. pseudo-classes like `:hover` — planned for a later batch).
            // Recover by discarding everything up to and including the next
            // block boundary as one unit, so leftover tokens (like a stray
            // "hover" identifier) can never be misread as the start of a new
            // rule. Without this, an unsupported selector silently corrupts
            // the parse instead of just being dropped.
            self.recover_malformed_rule();
            return None;
        }
        self.pos += 1; // consume '{'
        let declarations = self.parse_declarations();
        Some(Rule {
            selectors,
            declarations,
        })
    }

    fn recover_malformed_rule(&mut self) {
        loop {
            match self.peek() {
                None | Some(CssToken::Eof) => return,
                Some(CssToken::LeftBrace) => {
                    self.skip_balanced_block();
                    return;
                }
                Some(CssToken::Semicolon) | Some(CssToken::RightBrace) => {
                    self.pos += 1;
                    return;
                }
                _ => {
                    self.pos += 1;
                }
            }
        }
    }

    fn parse_selector_list(&mut self) -> Option<Vec<Selector>> {
        let mut selectors = Vec::new();
        loop {
            self.skip_whitespace();
            let selector = self.parse_single_selector()?;
            selectors.push(selector);
            self.skip_whitespace();
            match self.peek() {
                Some(CssToken::Comma) => {
                    self.pos += 1;
                }
                _ => break,
            }
        }
        Some(selectors)
    }

    fn parse_single_selector(&mut self) -> Option<Selector> {
        let mut steps = Vec::new();
        let first = self.parse_simple_selector()?;
        steps.push((None, first));

        loop {
            let had_whitespace = matches!(self.peek(), Some(CssToken::Whitespace));
            if had_whitespace {
                self.pos += 1;
                self.skip_whitespace();
            }

            let explicit_combinator = match self.peek() {
                Some(CssToken::Delim('>')) => Some(Combinator::Child),
                Some(CssToken::Delim('+')) => Some(Combinator::NextSibling),
                Some(CssToken::Delim('~')) => Some(Combinator::SubsequentSibling),
                _ => None,
            };

            if let Some(comb) = explicit_combinator {
                self.pos += 1;
                self.skip_whitespace();
                match self.parse_simple_selector() {
                    Some(simple) => steps.push((Some(comb), simple)),
                    None => break, // trailing combinator with nothing after — stop
                }
            } else if had_whitespace && Self::starts_simple_selector(self.peek()) {
                match self.parse_simple_selector() {
                    Some(simple) => steps.push((Some(Combinator::Descendant), simple)),
                    None => break,
                }
            } else {
                break;
            }
        }

        Some(Selector { steps })
    }

    fn starts_simple_selector(tok: Option<&CssToken>) -> bool {
        matches!(
            tok,
            Some(CssToken::Ident(_)) | Some(CssToken::Hash(_)) | Some(CssToken::Delim('.'))
                | Some(CssToken::Delim('*'))
        )
    }

    fn parse_simple_selector(&mut self) -> Option<SimpleSelector> {
        let mut simple = SimpleSelector::default();
        let mut matched_anything = false;

        if let Some(CssToken::Delim('*')) = self.peek() {
            simple.universal = true;
            self.pos += 1;
            matched_anything = true;
        } else if let Some(CssToken::Ident(name)) = self.peek() {
            simple.tag = Some(name.to_ascii_lowercase());
            self.pos += 1;
            matched_anything = true;
        }

        loop {
            match self.peek() {
                Some(CssToken::Hash(id)) => {
                    simple.id = Some(id.clone());
                    self.pos += 1;
                    matched_anything = true;
                }
                Some(CssToken::Delim('.')) => {
                    self.pos += 1;
                    if let Some(CssToken::Ident(class)) = self.peek() {
                        simple.classes.push(class.clone());
                        self.pos += 1;
                        matched_anything = true;
                    } else {
                        // malformed "." with nothing after — recover by stopping
                        break;
                    }
                }
                _ => break,
            }
        }

        if matched_anything {
            Some(simple)
        } else {
            None
        }
    }

    fn parse_declarations(&mut self) -> Vec<Declaration> {
        let mut decls = Vec::new();
        loop {
            self.skip_whitespace();
            match self.peek() {
                None | Some(CssToken::Eof) | Some(CssToken::RightBrace) => {
                    if matches!(self.peek(), Some(CssToken::RightBrace)) {
                        self.pos += 1;
                    }
                    break;
                }
                Some(CssToken::Semicolon) => {
                    self.pos += 1; // stray semicolon, e.g. empty declaration
                }
                Some(CssToken::Ident(_)) => {
                    if let Some(decl) = self.parse_one_declaration() {
                        decls.push(decl);
                    }
                }
                _ => {
                    // malformed token where a property name was expected —
                    // skip it and keep going rather than aborting the block
                    self.pos += 1;
                }
            }
        }
        decls
    }

    fn parse_one_declaration(&mut self) -> Option<Declaration> {
        let property = match self.advance() {
            Some(CssToken::Ident(name)) => name.to_ascii_lowercase(),
            _ => return None,
        };
        self.skip_whitespace();
        if !matches!(self.peek(), Some(CssToken::Colon)) {
            // malformed: "property" with no colon — skip to next ';' or '}'
            self.recover_to_declaration_end();
            return None;
        }
        self.pos += 1; // consume ':'
        self.skip_whitespace();

        let mut value_tokens = Vec::new();
        loop {
            match self.peek() {
                None | Some(CssToken::Eof) | Some(CssToken::Semicolon)
                | Some(CssToken::RightBrace) => break,
                Some(tok) => {
                    value_tokens.push(tok.clone());
                    self.pos += 1;
                }
            }
        }
        if matches!(self.peek(), Some(CssToken::Semicolon)) {
            self.pos += 1;
        }

        let important = Self::strip_important(&mut value_tokens);
        let value = Self::render_value(&value_tokens);

        Some(Declaration {
            property,
            value,
            important,
        })
    }

    fn recover_to_declaration_end(&mut self) {
        loop {
            match self.peek() {
                None | Some(CssToken::Eof) | Some(CssToken::RightBrace) => return,
                Some(CssToken::Semicolon) => {
                    self.pos += 1;
                    return;
                }
                _ => {
                    self.pos += 1;
                }
            }
        }
    }

    /// Detects a trailing "!important" in a declaration's value tokens and
    /// removes it, returning whether it was present.
    fn strip_important(tokens: &mut Vec<CssToken>) -> bool {
        // trim trailing whitespace first
        while matches!(tokens.last(), Some(CssToken::Whitespace)) {
            tokens.pop();
        }
        if let Some(CssToken::Ident(word)) = tokens.last() {
            if word.eq_ignore_ascii_case("important") {
                let ident_pos = tokens.len() - 1;
                // look back through optional whitespace for the '!' delim
                let mut i = ident_pos;
                while i > 0 && matches!(tokens[i - 1], CssToken::Whitespace) {
                    i -= 1;
                }
                if i > 0 && matches!(tokens[i - 1], CssToken::Delim('!')) {
                    tokens.truncate(i - 1);
                    while matches!(tokens.last(), Some(CssToken::Whitespace)) {
                        tokens.pop();
                    }
                    return true;
                }
            }
        }
        false
    }

    fn render_value(tokens: &[CssToken]) -> String {
        let mut out = String::new();
        for tok in tokens {
            match tok {
                CssToken::Whitespace => out.push(' '),
                CssToken::Ident(s) => out.push_str(s),
                CssToken::Str(s) => {
                    out.push('"');
                    out.push_str(s);
                    out.push('"');
                }
                CssToken::Number(n) => out.push_str(&n.to_string()),
                CssToken::Percentage(n) => {
                    out.push_str(&n.to_string());
                    out.push('%');
                }
                CssToken::Hash(h) => {
                    out.push('#');
                    out.push_str(h);
                }
                CssToken::Delim(c) => out.push(*c),
                CssToken::Comma => out.push(','),
                CssToken::LeftParen => out.push('('),
                CssToken::RightParen => out.push(')'),
                _ => {}
            }
        }
        out.trim().to_string()
    }
}