//! instruct model

use chrono::Local;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Default, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "instruct")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub data: String,
    pub name: String,
    pub des: String,
    pub created_by: String,
    pub updated_by: String,
    pub created_at: chrono::DateTime<Local>,
    pub updated_at: chrono::DateTime<Local>,
    pub deleted: i8,
}
impl Model {
    pub fn new() -> Self {
        let now_time = Local::now();
        Model {
            id: "".to_string(),
            data: Default::default(),
            name: Default::default(),
            des: Default::default(),
            created_by: Default::default(),
            updated_by: Default::default(),
            created_at: now_time,
            updated_at: now_time,
            deleted: 0,
        }
    }
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}