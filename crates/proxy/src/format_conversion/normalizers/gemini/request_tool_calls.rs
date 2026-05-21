use std::collections::{HashMap, VecDeque};

#[derive(Default)]
pub(super) struct GeminiToolCallTracker {
    by_name: HashMap<String, VecDeque<String>>,
}

impl GeminiToolCallTracker {
    pub(super) fn push(&mut self, name: String, id: String) {
        self.by_name.entry(name).or_default().push_back(id);
    }

    pub(super) fn pop(&mut self, name: &str) -> String {
        self.by_name.get_mut(name).and_then(VecDeque::pop_front).unwrap_or_else(|| name.to_owned())
    }

    pub(super) fn synthetic_id(&mut self, name: &str) -> String {
        if name.is_empty() {
            return "toolu_0".to_owned();
        }
        format!("toolu_{name}")
    }
}
