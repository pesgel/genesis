use chrono::Local;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeVO {
    pub id: String,
    pub name: String,
    pub host: String,
    pub account: String,
    pub port: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeListItemVO {
    pub id: String,
    pub name: String,
    pub host: String,
    pub account: String,
    pub port: u32,
    pub created_by: String,
    pub updated_by: String,
    pub created_at: chrono::DateTime<Local>,
    pub updated_at: chrono::DateTime<Local>,
}
