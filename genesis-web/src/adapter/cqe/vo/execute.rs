use chrono::Local;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteVO {
    pub id: String,
    pub state: i32,
    pub name: String,
    pub node_id: String,
    pub remark: String,
    pub replaces: String,
    pub node_name: String,
    pub instruct_id: String,
    pub instruct_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteListItemVO {
    pub id: String,
    pub state: i32,
    pub name: String,
    pub node_id: String,
    pub node_name: String,
    pub remark: String,
    pub replaces: String,
    pub instruct_id: String,
    pub instruct_name: String,
    pub created_by: String,
    pub updated_by: String,
    pub created_at: chrono::DateTime<Local>,
    pub updated_at: chrono::DateTime<Local>,
}
