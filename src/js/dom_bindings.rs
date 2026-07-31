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
