#![allow(dead_code)]

pub mod ast;
pub mod bytecode;
pub mod dom_bindings;
pub mod gc;
pub mod lexer;
pub mod parser;
pub mod vm;
pub mod worker;

pub use bytecode::{Compiler, Value};
pub use dom_bindings::execute_js_on_dom;
pub use lexer::JsLexer;
pub use parser::JsParser;
pub use vm::VM;
pub use worker::WebWorker;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::{build_dom, NodeType};
    use crate::html::Tokenizer;

    #[test]
    fn test_lexer_and_parser() {
        let code = "var x = 10 + 20; if (x > 15) { x = 100; }";
        let tokens = JsLexer::new(code).tokenize();
        let program = JsParser::new(tokens).parse();
        assert_eq!(program.body.len(), 2);
    }

    #[test]
    fn test_vm_execution() {
        let code = "var a = 5; var b = 10; var c = a + b;";
        let tokens = JsLexer::new(code).tokenize();
        let program = JsParser::new(tokens).parse();
        let chunk = Compiler::new().compile(&program);
        let mut vm = VM::new();
        vm.run(&chunk);
        assert_eq!(vm.globals.get("c"), Some(&Value::Number(15.0)));
    }

    #[test]
    fn test_dom_mutation_from_js() {
        let html = r#"<html><body><h1 id="header">Old Title</h1></body></html>"#;
        let tokens = Tokenizer::new(html).tokenize();
        let mut dom = build_dom(tokens);

        let script = r#"setElementText("header", "New Updated Title");"#;
        execute_js_on_dom(script, &mut dom);

        let mut found_updated_text = false;
        for node in &dom.nodes {
            if let NodeType::Text(t) = &node.node_type {
                if t == "New Updated Title" {
                    found_updated_text = true;
                    break;
                }
            }
        }
        assert!(found_updated_text);
    }

    #[test]
    fn test_web_apis_timers_events_fetch() {
        let html = r#"<html><body><h1 id="header">Title</h1></body></html>"#;
        let mut dom = build_dom(Tokenizer::new(html).tokenize());

        let script = r#"
            addEventListener("click", 1);
            setTimeout(1, 100);
        "#;
        execute_js_on_dom(script, &mut dom);
    }

    #[test]
    fn test_canvas_2d_api() {
        let html = r#"<html><body><canvas id="myCanvas" width="200" height="100"></canvas></body></html>"#;
        let mut dom = build_dom(Tokenizer::new(html).tokenize());

        let script = r#"
            getContext("2d");
            fillRect(10, 20, 100, 50);
        "#;
        execute_js_on_dom(script, &mut dom);
    }

    #[test]
    fn test_web_worker_multithreading() {
        let worker = WebWorker::new("worker.js");
        assert!(worker.post_message("task_data").is_ok());
        std::thread::sleep(std::time::Duration::from_millis(50));
        let response = worker.try_recv();
        assert_eq!(response, Some("processed: task_data".to_string()));
    }
}

