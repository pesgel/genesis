use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Envelope {
    pub version: String,
    pub r#type: String,
    pub payload: String,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageQuery {
    page: u64,
    size: u64,
}
impl PageQuery {
    pub fn init(&self) -> (u64, u64) {
        (self.page, self.size)
    }
}
