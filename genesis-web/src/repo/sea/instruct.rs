//! instruct repo
use crate::repo::model::instruct;
use crate::repo::sea::SeaRepo;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DbConn, DbErr, EntityTrait};

pub struct InstructRepo;

impl InstructRepo {
    pub async fn update_instruct_by_id(
        db: &DbConn,
        model: instruct::Model,
    ) -> Result<instruct::Model, DbErr> {
        instruct::ActiveModel {
            id: Set(model.id),
            name: Set(model.name),
            des: Set(model.des),
            data: Set(model.data),
            ..Default::default()
        }
        .update(db)
        .await
    }

    pub async fn insert_instruct_one(
        db: &DbConn,
        data: instruct::ActiveModel,
    ) -> Result<(), DbErr> {
        let res = instruct::Entity::insert(data).exec(db).await;
        match res {
            Ok(_) => Ok(()),
            Err(DbErr::RecordNotInserted) => Ok(()),
            Err(e) => Err(e),
        }
    }
    pub async fn get_instruct_by_id(db: &DbConn, id: &str) -> Result<instruct::Model, DbErr> {
        instruct::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("not found".to_string()))
    }

    pub async fn find_instruct_by(
        db: &DbConn,
        pg: (u64, u64),
    ) -> anyhow::Result<(u64, Vec<instruct::Model>)> {
        SeaRepo::page_with_default::<instruct::Entity>(db, pg, None).await
    }
}
