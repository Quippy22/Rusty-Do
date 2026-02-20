use crate::models::task::Task;

#[derive(Clone)]
pub struct Notebook {
    pub name: String,
    pub tasks: Vec<Task>,
}
