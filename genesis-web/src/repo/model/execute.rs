use chrono::Local;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Default, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "execute_task")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub name: String,
    pub node_id: String,
    pub node_name: String,
    pub state: i32,
    pub remark: String,
    pub replaces: String,
    pub instruct_id: String,
    pub instruct_name: String,
    pub created_by: String,
    pub updated_by: String,
    pub created_at: chrono::DateTime<Local>,
    pub updated_at: chrono::DateTime<Local>,
    pub deleted: i8,
}

impl Model {
    pub fn new() -> Model {
        Model::default()
    }
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
