//! instruct

use genesis_process::InData;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")] // 使用驼峰命名格式
pub struct InstructSaveCmd {
    pub id: Option<String>,
    pub des: Option<String>,
    pub name: String,
    pub data: InData,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")] // 使用驼峰命名格式
pub struct InstructExecuteCmd {
    pub id: String,
    pub name: String,
    pub node: String,
}
