use crate::common::AuthType;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CredentialSaveCmd {
    pub id: Option<String>,
    pub principal: String,
    pub credential: String,
    pub asset_id: String,
    pub protocol_id: String,
    pub auth_type: AuthType,
    pub address: String,
    pub protocol: String,
    pub port: i32,
    pub remark: Option<String>,
}
