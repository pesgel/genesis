use crate::common::EnvelopeType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Envelope {
    pub version: String,
    pub r#type: EnvelopeType,
    pub payload: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageQuery {
    page: u64,
    size: u64,
    sort: Option<String>,
    sort_by: Option<String>,
}
impl PageQuery {
    pub fn init(&self) -> (u64, u64) {
        (self.page, self.size)
    }
}
