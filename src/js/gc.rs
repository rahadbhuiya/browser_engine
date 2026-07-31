use std::collections::HashMap;
use super::bytecode::Value;

pub type GcId = usize;

#[derive(Debug, Clone)]
pub struct GcObject {
    pub properties: HashMap<String, Value>,
    pub marked: bool,
}

pub struct GarbageCollector {
    pub objects: HashMap<GcId, GcObject>,
    next_id: GcId,
}

impl GarbageCollector {
    pub fn new() -> Self {
        GarbageCollector {
            objects: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn alloc_object(&mut self) -> GcId {
        let id = self.next_id;
        self.next_id += 1;
        self.objects.insert(
            id,
            GcObject {
                properties: HashMap::new(),
                marked: false,
            },
        );
        id
    }

    pub fn sweep(&mut self) {
        self.objects.retain(|_, obj| {
            if obj.marked {
                obj.marked = false;
                true
            } else {
                false
            }
        });
    }
}
