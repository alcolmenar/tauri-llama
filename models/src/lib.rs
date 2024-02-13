use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub struct Conversation {
    pub messages: Vec<Message>,
}

impl Conversation {
    pub fn new() -> Conversation {
        Conversation {
            messages: Vec::new(),
        }
    }
}

impl Default for Conversation {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub user: bool,
    pub text: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NewTokenRes {
    pub token: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ModelPath {
    pub path: String,
}
