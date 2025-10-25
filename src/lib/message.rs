use serde::{Deserialize, Serialize};

use crate::models::{CommentView, TaskView};
use crate::query::{PagenatedTaskList, TaskQuery};


#[derive(Serialize)]
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

#[derive(Serialize)]
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

#[derive(Serialize)]
pub struct PagenatedTaskListErrorMessage {
    message: String,
}

impl PagenatedTaskListErrorMessage {
    pub fn message(&self) -> &String {
        &(self.message)
    }
}

#[derive(Serialize)]
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

#[derive(Serialize)]
pub struct TaskInfoMessage {
    message: String,
    task: TaskView,
}

impl TaskInfoMessage {
    pub fn message(&self) -> &String {
        &(self.message)
    }

    pub fn task(&self) -> &TaskView {
        &(self.task)
    }
}

#[derive(Serialize)]
pub struct TaskErrorMessage {
    message: String,
}

impl TaskErrorMessage {
    pub fn message(&self) -> &String {
        &(self.message)
    }
}

#[derive(Serialize)]
pub enum TaskMessage {
    Info(TaskInfoMessage),
    Error(TaskErrorMessage),
}

impl TaskMessage {
    pub fn info(message: String, task: TaskView) -> Self {
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

#[derive(Serialize)]
pub struct CommentInfoMessage {
    message: String,
    comment: CommentView,
}

impl CommentInfoMessage {
    pub fn message(&self) -> &String {
        &(self.message)
    }

    pub fn comment(&self) -> &CommentView {
        &(self.comment)
    }
}

#[derive(Serialize)]
pub struct CommentErrorMessage {
    message: String,
}

impl CommentErrorMessage {
    pub fn message(&self) -> &String {
        &(self.message)
    }
}

#[derive(Serialize)]
pub enum CommentMessage {
    Info(CommentInfoMessage),
    Error(CommentErrorMessage),
}

impl CommentMessage {
    pub fn info(message: String, comment: CommentView) -> Self {
        CommentMessage::Info(CommentInfoMessage {
            message,
            comment,
        })
    }

    pub fn error(message: String) -> Self {
        CommentMessage::Error(CommentErrorMessage {
            message,
        })
    }
}
