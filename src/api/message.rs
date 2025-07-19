use serde::{Deserialize, Serialize};

use super::models::Task;


#[derive(Deserialize, Serialize)]
pub struct TaskInfoMessage {
    message: String,
    task: Task,
}

impl TaskInfoMessage {
    pub fn message(&self) -> &String {
        &(self.message)
    }

    pub fn task(&self) -> &Task {
        &(self.task)
    }
}

#[derive(Deserialize, Serialize)]
pub struct TaskErrorMessage {
    message: String,
}

impl TaskErrorMessage {
    pub fn message(&self) -> &String {
        &(self.message)
    }
}

#[derive(Deserialize, Serialize)]
pub enum TaskMessage {
    Info(TaskInfoMessage),
    Error(TaskErrorMessage),
}

impl TaskMessage {
    pub fn info(message: String, task: Task) -> Self {
        TaskMessage::Info(TaskInfoMessage {
            message,
            task,
        })
    }

    pub fn error(message: String) -> Self {
        TaskMessage::Error(TaskErrorMessage {
            message,
        })
    }
}
