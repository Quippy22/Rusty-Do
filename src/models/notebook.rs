use crate::models::task::Task;

pub struct Notebook {
    pub name: String,
    pub tasks: Vec<Task>,
}
