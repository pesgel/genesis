use chrono::Local;
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetVO {
    pub id: String,
    pub name: String,
    pub org_id: String,
    pub address: String,
    pub location: String,
    pub alias_name: String,
    pub asset_type: String,
    pub address_type: String,
    pub status: i32,
    pub remark: String,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetListItemVO {
    pub id: String,
    pub name: String,
    pub org_id: String,
    pub address: String,
    pub location: String,
    pub alias_name: String,
    pub asset_type: String,
    pub address_type: String,
    pub status: i32,
    pub remark: String,
    pub created_by: String,
    pub updated_by: String,
    pub created_at: chrono::DateTime<Local>,
    pub updated_at: chrono::DateTime<Local>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, FromQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct AssetProtocolListItemVO {
    pub asset_id: String,
    pub asset_name: String,
    pub address: String,
    pub protocol_id: String,
    pub protocol: String,
    pub port: i32,
}
