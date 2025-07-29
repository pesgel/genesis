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
    pub fn order_clause(&self) -> Option<String> {
        // 尝试解析用户输入
        if let Some(sort) = self.sort.as_ref() {
            if sort.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                let sort_by = self.sort_by.as_deref().unwrap_or("ASC").to_uppercase();
                if sort_by == "ASC" || sort_by == "DESC" {
                    return Some(format!("{sort} {sort_by}"));
                }
            }
        }
        None
    }
}
