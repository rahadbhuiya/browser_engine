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
            let event_type = args[0].to_string();
            let handler_desc = args[1].to_string();
            let use_capture = if args.len() >= 3 {
                match &args[2] {
                    Value::Boolean(b) => *b,
                    _ => false,
                }
            } else {
                false
            };
            println!(
                "JS Event: Registered listener for '{}' (capture={}) -> {}",
                event_type, use_capture, handler_desc
            );
        }
        Value::Undefined
    });

    vm.register_native_fn("removeEventListener", move |_vm, args| {
        if !args.is_empty() {
            println!("JS Event: Removed listener for '{}'", args[0].to_string());
        }
        Value::Undefined
    });

    vm.register_native_fn("dispatchEvent", move |_vm, args| {
        if !args.is_empty() {
            let ev_type = args[0].to_string();
            println!("JS Event: Dispatched custom event '{}'", ev_type);
            return Value::Boolean(true);
        }
        Value::Boolean(false)
    });

    vm.register_native_fn("submitForm", move |_vm, args| {
        if !args.is_empty() {
            let target_id = args[0].to_string();
            unsafe {
                let dom_ref = &mut *(dom_ptr_val as *mut Dom);
                if let Some(idx) = dom_ref.get_element_by_id(&target_id) {
                    println!("JS Form: Submitting form #{}", target_id);
                    let mgr = crate::dom::FormManager::new();
                    if let Ok(payload) = crate::dom::build_form_submission(dom_ref, idx, &mgr, "https://example.com/") {
                        return Value::String(payload.target_url);
                    }
                }
            }
        }
        Value::Undefined
    });

    vm.register_native_fn("playMedia", move |_vm, args| {
        if !args.is_empty() {
            let target_id = args[0].to_string();
            println!("JS Media: play() called on #{}", target_id);
            return Value::Boolean(true);
        }
        Value::Boolean(false)
    });

    vm.register_native_fn("pauseMedia", move |_vm, args| {
        if !args.is_empty() {
            let target_id = args[0].to_string();
            println!("JS Media: pause() called on #{}", target_id);
            return Value::Boolean(true);
        }
        Value::Boolean(false)
    });

    vm.register_native_fn("seekMedia", move |_vm, args| {
        if args.len() >= 2 {
            let target_id = args[0].to_string();
            let target_time = match &args[1] {
                Value::Number(n) => *n,
                _ => 0.0,
            };
            println!("JS Media: seek(#{}, {}s)", target_id, target_time);
            return Value::Number(target_time);
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

    vm.register_native_fn("Worker", move |_vm, args| {
        if !args.is_empty() {
            let script_url = args[0].to_string();
            println!("WebWorker: Spawning worker thread for {}", script_url);
            return Value::String(format!("Worker({})", script_url));
        }
        Value::Undefined
    });

    vm.register_native_fn("postMessage", move |_vm, args| {
        if !args.is_empty() {
            println!("WebWorker postMessage: {}", args[0].to_string());
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
