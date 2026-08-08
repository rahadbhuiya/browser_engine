use crate::dom::{Dom, NodeType};
use super::bytecode::{Compiler, Value};
use super::lexer::JsLexer;
use super::parser::JsParser;
use super::vm::VM;

pub fn execute_js_on_dom(script: &str, dom: &mut Dom) {
    let tokens = JsLexer::new(script).tokenize();
    let program = JsParser::new(tokens).parse();
    let chunk = Compiler::new().compile(&program);

    let mut vm = VM::new();
    let dom_ptr_val = dom as *mut Dom as usize;

    vm.register_native_fn("addEventListener", move |_vm, args| {
        if args.len() >= 2 {
            println!("JS Event: Registered listener for '{}'", args[0].to_string());
        }
        Value::Undefined
    });

    vm.register_native_fn("setTimeout", move |_vm, args| {
        if args.len() >= 2 {
            println!("JS Timer: Scheduled callback for {} ms", args[1].to_string());
        }
        Value::Number(1.0)
    });

    vm.register_native_fn("fetch", move |_vm, args| {
        if !args.is_empty() {
            let url = args[0].to_string();
            println!("JS fetch API requested: {}", url);
            if let Ok(resp) = crate::net::fetch(&url) {
                return Value::String(resp.body_as_string());
            }
        }
        Value::Undefined
    });

    vm.register_native_fn("getContext", move |_vm, args| {
        if !args.is_empty() && args[0].to_string() == "2d" {
            println!("Canvas 2D: Context 2D initialized");
            return Value::String("CanvasRenderingContext2D".to_string());
        }
        Value::Undefined
    });

    vm.register_native_fn("fillRect", move |_vm, args| {
        if args.len() >= 4 {
            println!(
                "Canvas 2D fillRect(x={}, y={}, w={}, h={})",
                args[0].to_string(),
                args[1].to_string(),
                args[2].to_string(),
                args[3].to_string()
            );
        }
        Value::Undefined
    });

    vm.register_native_fn("setElementText", move |_vm, args| {
        if args.len() >= 2 {
            let target_id = args[0].to_string();
            let new_text = args[1].to_string();
            unsafe {
                let dom_ref = &mut *(dom_ptr_val as *mut Dom);
                for i in 0..dom_ref.nodes.len() {
                    let is_match = match &dom_ref.nodes[i].node_type {
                        NodeType::Element(el) => el.id.as_deref() == Some(&target_id),
                        _ => false,
                    };
                    if is_match {
                        if !dom_ref.nodes[i].children.is_empty() {
                            let child_idx = dom_ref.nodes[i].children[0];
                            dom_ref.nodes[child_idx].node_type = NodeType::Text(new_text.clone());
                        } else {
                            let new_idx = dom_ref.nodes.len();
                            dom_ref.nodes.push(crate::dom::DomNode {
                                node_type: NodeType::Text(new_text.clone()),
                                parent: Some(i),
                                children: vec![],
                            });
                            dom_ref.nodes[i].children.push(new_idx);
                        }
                        break;
                    }
                }
            }
        }
        Value::Undefined
    });

    vm.run(&chunk);
}
