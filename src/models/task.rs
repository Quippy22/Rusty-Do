use crate::models::subtask::Subtask;

#[derive(Clone)]
pub struct Task {
    pub name: String, 
    pub description: String,
    pub completion: f32,
    pub is_done: bool,
    pub subtasks: Vec<Subtask>,
}