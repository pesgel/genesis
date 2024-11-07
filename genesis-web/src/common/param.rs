use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Envelope {
    pub version: String,
    pub r#type: String,
    pub payload: String,
}
