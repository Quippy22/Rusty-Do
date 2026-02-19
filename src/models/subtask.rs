pub struct Subtask {
    pub name: String,
    pub is_done: bool,
}

impl Subtask {
    pub fn mark_as_done(&mut self) {
        self.is_done = true;
    }
}