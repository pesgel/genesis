use serde::Serialize;

pub mod cmd;

// How we want errors responses to be serialized
#[derive(Clone, Serialize)]
pub struct Response {
    status: u16,
    message: String,
    timestamp: String,
}

impl Default for Response {
    fn default() -> Self {
        Self {
            status: 200,
            message: "success".to_string(),
            timestamp: chrono::Local::now().to_string(),
        }
    }
}
