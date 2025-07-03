use serde::{Deserialize, Serialize};

pub mod asset;
pub mod credential;
pub mod execute;
pub mod instruct;
pub mod node;
pub mod user;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BaseKV {
    pub key: String,
    pub value: String,
}
