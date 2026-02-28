use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Subtask {
    pub name: String,
    pub is_done: bool,
}

impl Subtask {
    pub fn toggle(&mut self) {
        self.is_done = !self.is_done;
    }
}
