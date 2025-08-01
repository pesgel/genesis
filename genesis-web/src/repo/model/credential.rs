use chrono::Local;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "asset_account")]
#[serde(default)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub principal: String,
    pub credential: String,
    pub asset_id: String,
    pub auth_type: String,
    pub protocol_id: String,
    pub address: String,
    pub protocol: String,
    pub port: i32,
    pub status: i32,
    pub remark: String,
    pub created_by: String,
    pub updated_by: String,
    pub created_at: chrono::DateTime<Local>,
    pub updated_at: chrono::DateTime<Local>,
    pub deleted: i8,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn new() -> Model {
        Model::default()
    }
}
