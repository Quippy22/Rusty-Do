use serde::{Deserialize, Serialize};

use crate::models::task::Task;

use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct Notebook {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tasks: Vec<Task>,
}

impl Notebook {
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            tasks: Vec::new(),
        }
    }
}
