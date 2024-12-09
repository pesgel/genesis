//! instruct repo

use crate::repo::model::instruct;
use sea_orm::{DbConn, DbErr, EntityTrait};

pub struct InstructRepo;

impl InstructRepo {
    pub async fn insert_instruct_one(
        db: &DbConn,
        data: instruct::ActiveModel,
    ) -> Result<(), String> {
        let res = instruct::Entity::insert(data).exec(db).await;
        match res {
            Ok(_) => Ok(()),
            Err(DbErr::RecordNotInserted) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
    pub async fn get_instruct_by_id(db: &DbConn, id: &str) -> Result<instruct::Model, String> {
        instruct::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("get_instruct_by_id not found".to_string())
    }

    pub async fn find_instruct_by(db: &DbConn) -> Result<Vec<instruct::Model>, String> {
        instruct::Entity::find()
            .all(db)
            .await
            .map_err(|e| e.to_string())
    }
}
