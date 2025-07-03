use chrono::Local;
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialVO {
    pub id: String,
    pub principal: String,
    pub credential: String,
    pub asset_id: String,
    pub auth_type: String,
    pub protocol_id: String,
    pub status: i32,
    pub remark: String,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, FromQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct CredentialListItemVO {
    pub id: String,
    pub principal: String,
    pub port: i32,
    pub address: String,
    pub asset_name: String,
    pub protocol: String,
    pub asset_id: String,
    pub auth_type: String,
    pub protocol_id: String,
    pub status: i32,
    pub remark: String,
    pub created_by: String,
    pub updated_by: String,
    pub created_at: chrono::DateTime<Local>,
    pub updated_at: chrono::DateTime<Local>,
}
