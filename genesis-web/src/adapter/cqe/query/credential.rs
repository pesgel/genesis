use crate::common::PageQuery;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialListQuery {
    pub page_query: PageQuery,
    pub principal: Option<String>,
    pub address: Option<String>,
    pub protocol: Option<String>,
    pub asset_name: Option<String>,
    pub auth_type: Option<String>,
}
