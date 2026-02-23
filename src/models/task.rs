use crate::models::subtask::Subtask;

#[derive(Clone)]
pub struct Task {
    pub name: String,
    pub description: String,
    pub completion: f32,
    pub is_done: bool,
    pub subtasks: Vec<Subtask>,
}

impl Task {
    pub fn toggle_subtask(&mut self, index: usize) {
        if let Some(subtask) = self.subtasks.get_mut(index) {
            subtask.toggle();
            self.recalculate_completion();
        }
    }

    pub fn toggle_task(&mut self) {
        self.is_done = !self.is_done;
        if self.is_done {
            self.completion = 100.0;
            for subtask in &mut self.subtasks {
                subtask.is_done = true;
            }
        } else {
            self.completion = 0.0;
            for subtask in &mut self.subtasks {
                subtask.is_done = false;
            }
        }
    }

    pub fn recalculate_completion(&mut self) {
        if self.subtasks.is_empty() {
            // If there are no subtasks, completion depends solely on is_done
            self.completion = if self.is_done { 100.0 } else { 0.0 };
            return;
        }

        let done_count = self.subtasks.iter().filter(|s| s.is_done).count();
        let percentage = (done_count as f32 / self.subtasks.len() as f32) * 100.0;
        self.completion = (percentage * 10.0).round() / 10.0;

        // If completion reaches 100%, the task is done
        self.is_done = self.completion == 100.0;
    }
}
