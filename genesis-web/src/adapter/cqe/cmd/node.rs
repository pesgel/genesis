use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")] // 使用驼峰命名格式
pub struct NodeSaveCmd {
    pub id: Option<String>,
    pub name: String,
    pub host: String,
    pub port: i32,
    pub account: String,
    pub password: String,
    pub remark: String,
}
