use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize)]
pub struct Task {
    pub title: String,
}

impl Task {
    pub fn new(title: String) -> Self {
        Task {
            title
        }
    }
}

#[derive(Serialize)]
pub struct TaskList {
    tasks: Vec<Task>,
}

impl TaskList {
    pub fn new(tasks: Vec<Task>) -> Self {
        TaskList {
            tasks
        }
    }
}
