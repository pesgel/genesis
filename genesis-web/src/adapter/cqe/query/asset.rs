use crate::common::PageQuery;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetListQuery {
    pub page_query: PageQuery,
    pub name: Option<String>,
    pub alias_name: Option<String>,
    pub asset_type: Option<String>,
    pub address: Option<String>,
    pub address_type: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetProtocolListQuery {
    pub page_query: PageQuery,
    pub asset_name: Option<String>,
}
