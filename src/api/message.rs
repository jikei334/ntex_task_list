use serde::{Deserialize, Serialize};

use crate::api::models::Task;
use crate::api::query::{PagenatedTaskList, TaskQuery};


#[derive(Deserialize, Serialize)]
pub enum SimpleMessage {
    Info(String),
    Error(String),
}

impl SimpleMessage {
    pub fn info(message: String) -> Self {
        SimpleMessage::Info(message)
    }

    pub fn error(message: String) -> Self {
        SimpleMessage::Error(message)
    }
}

#[derive(Deserialize, Serialize)]
pub struct PagenatedTaskListInfoMessage {
    message: String,
    pagenated_task_list: PagenatedTaskList,
}

impl PagenatedTaskListInfoMessage {
    pub fn message(&self) -> &String {
        &(self.message)
    }

    pub fn pagenated_task_list(&self) -> &PagenatedTaskList {
        &(self.pagenated_task_list)
    }
}

#[derive(Deserialize, Serialize)]
pub struct PagenatedTaskListErrorMessage {
    message: String,
}

impl PagenatedTaskListErrorMessage {
    pub fn message(&self) -> &String {
        &(self.message)
    }
}

#[derive(Deserialize, Serialize)]
pub enum PagenatedTaskListMessage {
    Info(PagenatedTaskListInfoMessage),
    Error(PagenatedTaskListErrorMessage),
}

impl PagenatedTaskListMessage {
    pub fn info(message: String, pagenated_task_list: PagenatedTaskList) -> Self {
        PagenatedTaskListMessage::Info(PagenatedTaskListInfoMessage {
            message,
            pagenated_task_list,
        })
    }

    pub fn error(message: String) -> Self {
        PagenatedTaskListMessage::Error(PagenatedTaskListErrorMessage {
            message,
        })
    }
}

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
