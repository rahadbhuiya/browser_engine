use crate::dom::{Dom, NodeType};
use crate::js::bytecode::Compiler;
use crate::js::dom_bindings::execute_js_on_dom;
use crate::js::lexer::JsLexer;
use crate::js::parser::JsParser;
use crate::js::vm::VM;

pub struct JsRuntime {
    pub vm: VM,
    pub console_logs: Vec<String>,
}

impl JsRuntime {
    pub fn new() -> Self {
        JsRuntime {
            vm: VM::new(),
            console_logs: Vec::new(),
        }
    }

    /// Evaluates a JavaScript code snippet and executes it on the live DOM tree
    pub fn execute(&mut self, script: &str, dom: &mut Dom) -> Result<(), String> {
        let trimmed = script.trim();
        if trimmed.is_empty() {
            return Ok(());
        }

        println!("⚡ [JsRuntime] Executing script ({} bytes)", trimmed.len());

        let tokens = JsLexer::new(trimmed).tokenize();
        let program = JsParser::new(tokens).parse();
        let chunk = Compiler::new().compile(&program);

        execute_js_on_dom(trimmed, dom);
        self.vm.run(&chunk);

        Ok(())
    }

    /// Helper to extract all script tags from the DOM tree
    pub fn extract_scripts(dom: &Dom) -> Vec<String> {
        let mut scripts = Vec::new();
        for node in &dom.nodes {
            if let NodeType::Element(elem) = &node.node_type {
                if elem.tag.eq_ignore_ascii_case("script") {
                    for child_idx in &node.children {
                        if let NodeType::Text(text) = &dom.nodes[*child_idx].node_type {
                            let s = text.trim();
                            if !s.is_empty() {
                                scripts.push(s.to_string());
                            }
                        }
                    }
                }
            }
        }
        scripts
    }
}
