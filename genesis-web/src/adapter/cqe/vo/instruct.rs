use chrono::Local;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstructVO {
    pub id: String,
    pub data: String,
    pub name: String,
    pub des: String,
    pub created_by: String,
    pub updated_by: String,
    pub created_at: chrono::DateTime<Local>,
    pub updated_at: chrono::DateTime<Local>,
}
