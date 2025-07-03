use crate::common::{AssetAddressType, AssetType};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AssetSaveCmd {
    pub name: String,
    pub address: String,
    pub asset_type: AssetType,
    pub address_type: AssetAddressType,
    pub remark: Option<String>,
    pub org_id: Option<String>,
    pub location: Option<String>,
    pub alias_name: Option<String>,
    pub protocol_list: Option<Vec<ProtocolSaveItem>>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolSaveItem {
    pub protocol: String,
    #[validate(range(min = 1, max = 65535, message = "port must in [1~65535]"))]
    pub port: i32,
}
