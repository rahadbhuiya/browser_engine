use std::collections::HashMap;
use crate::dom::Dom;

/// W3C DOM Event Flow phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventPhase {
    None = 0,
    Capturing = 1,
    AtTarget = 2,
    Bubbling = 3,
}

/// Standard DOM Event object supporting capture, bubbling, and cancellation.
#[derive(Debug, Clone)]
pub struct DomEvent {
    pub event_type: String,
    pub target: usize,
    pub current_target: usize,
    pub event_phase: EventPhase,
    pub bubbles: bool,
    pub cancelable: bool,
    pub default_prevented: bool,
    pub propagation_stopped: bool,
    pub immediate_propagation_stopped: bool,
    pub time_stamp: u64,
    pub detail: Option<String>,
}

impl DomEvent {
    pub fn new(event_type: impl Into<String>, target: usize, bubbles: bool, cancelable: bool) -> Self {
        DomEvent {
            event_type: event_type.into(),
            target,
            current_target: target,
            event_phase: EventPhase::None,
            bubbles,
            cancelable,
            default_prevented: false,
            propagation_stopped: false,
            immediate_propagation_stopped: false,
            time_stamp: 0,
            detail: None,
        }
    }

    pub fn prevent_default(&mut self) {
        if self.cancelable {
            self.default_prevented = true;
        }
    }

    pub fn stop_propagation(&mut self) {
        self.propagation_stopped = true;
    }

    pub fn stop_immediate_propagation(&mut self) {
        self.propagation_stopped = true;
        self.immediate_propagation_stopped = true;
    }

    pub fn is_default_prevented(&self) -> bool {
        self.default_prevented
    }
}

/// Callback signature for native Rust DOM event listeners.
pub type EventCallback = Box<dyn Fn(&mut DomEvent, &mut Dom) + Send + Sync>;

pub struct EventListener {
    pub id: usize,
    pub event_type: String,
    pub use_capture: bool,
    pub callback: EventCallback,
}

/// Manages event listener registrations and handles W3C event dispatch across the DOM.
#[derive(Default)]
pub struct EventRegistry {
    listeners: HashMap<usize, Vec<EventListener>>, // node_idx -> listeners
    next_id: usize,
    pub event_log: Vec<String>, // for debugging and testing lifecycle traces
}

impl EventRegistry {
    pub fn new() -> Self {
        EventRegistry {
            listeners: HashMap::new(),
            next_id: 1,
            event_log: Vec::new(),
        }
    }

    /// Registers an event listener on a DOM node.
    pub fn add_event_listener<F>(
        &mut self,
        node_idx: usize,
        event_type: &str,
        use_capture: bool,
        callback: F,
    ) -> usize
    where
        F: Fn(&mut DomEvent, &mut Dom) + Send + Sync + 'static,
    {
        let id = self.next_id;
        self.next_id += 1;

        let listener = EventListener {
            id,
            event_type: event_type.to_string(),
            use_capture,
            callback: Box::new(callback),
        };

        self.listeners.entry(node_idx).or_default().push(listener);
        id
    }

    /// Removes an event listener by its ID.
    pub fn remove_event_listener(&mut self, node_idx: usize, listener_id: usize) -> bool {
        if let Some(list) = self.listeners.get_mut(&node_idx) {
            let before = list.len();
            list.retain(|l| l.id != listener_id);
            list.len() < before
        } else {
            false
        }
    }

    /// Dispatches an event through the W3C 3-phase event flow (Capture -> Target -> Bubble).
    /// Returns `true` if the event completed without `preventDefault()`, `false` if cancelled.
    pub fn dispatch_event(&mut self, dom: &mut Dom, event: &mut DomEvent) -> bool {
        // Compute path from root to target
        let mut ancestors = dom.ancestors(event.target);
        ancestors.reverse(); // Now ordered: root -> ... -> parent of target

        // Phase 1: Capturing Phase (root down to parent)
        event.event_phase = EventPhase::Capturing;
        for &node_idx in &ancestors {
            if event.propagation_stopped {
                break;
            }
            event.current_target = node_idx;
            self.trigger_phase_listeners(node_idx, event, dom, true);
        }

        // Phase 2: Target Phase (target element itself)
        if !event.propagation_stopped {
            event.event_phase = EventPhase::AtTarget;
            event.current_target = event.target;
            self.trigger_target_listeners(event.target, event, dom);
        }

        // Phase 3: Bubbling Phase (parent up to root, if bubbles == true)
        if event.bubbles && !event.propagation_stopped {
            event.event_phase = EventPhase::Bubbling;
            ancestors.reverse(); // Now ordered: parent of target -> ... -> root
            for &node_idx in &ancestors {
                if event.propagation_stopped {
                    break;
                }
                event.current_target = node_idx;
                self.trigger_phase_listeners(node_idx, event, dom, false);
            }
        }

        !event.default_prevented
    }

    fn trigger_phase_listeners(
        &mut self,
        node_idx: usize,
        event: &mut DomEvent,
        dom: &mut Dom,
        capture_phase: bool,
    ) {
        if let Some(list) = self.listeners.get_mut(&node_idx) {
            let matching_indices: Vec<usize> = list
                .iter()
                .enumerate()
                .filter(|(_, l)| {
                    l.event_type.eq_ignore_ascii_case(&event.event_type)
                        && l.use_capture == capture_phase
                })
                .map(|(idx, _)| idx)
                .collect();

            for idx in matching_indices {
                if event.immediate_propagation_stopped {
                    break;
                }
                (list[idx].callback)(event, dom);
            }
        }
    }

    fn trigger_target_listeners(
        &mut self,
        node_idx: usize,
        event: &mut DomEvent,
        dom: &mut Dom,
    ) {
        if let Some(list) = self.listeners.get_mut(&node_idx) {
            let matching_indices: Vec<usize> = list
                .iter()
                .enumerate()
                .filter(|(_, l)| l.event_type.eq_ignore_ascii_case(&event.event_type))
                .map(|(idx, _)| idx)
                .collect();

            for idx in matching_indices {
                if event.immediate_propagation_stopped {
                    break;
                }
                (list[idx].callback)(event, dom);
            }
        }
    }
}
