use serde::{Deserialize, Serialize};


#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum Status {
    Info,
    Warn,
    Error,
}

#[derive(Deserialize, Serialize)]
pub struct Message {
    text: String,
    status: Status,
}

impl Message {
    pub fn info(text: String) -> Self {
        Message {
            text,
            status: Status::Info,
        }
    }

    pub fn text(&self) -> String {
        self.text.clone()
    }

    pub fn status(&self) -> Status {
        self.status
    }
}
