use crate::common::PageQuery;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteListQuery {
    pub page_query: PageQuery,
    pub name: Option<String>,
}
