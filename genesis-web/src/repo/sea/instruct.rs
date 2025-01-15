//! instruct repo
use crate::repo::model::instruct;
use crate::repo::sea::SeaRepo;
use sea_orm::sea_query::ConditionExpression;
use sea_orm::ActiveValue::Set;
use sea_orm::{DbConn, DbErr, EntityTrait};

pub struct InstructRepo;

impl InstructRepo {
    pub async fn save_instruct(db: &DbConn, model: instruct::Model) -> anyhow::Result<String> {
        if model.id.is_empty() {
            InstructRepo::insert_instruct_one(db, model).await
        } else {
            InstructRepo::update_instruct_by_id(db, model)
                .await
                .map(|data| anyhow::Ok(data.id))?
        }
    }
    pub async fn update_instruct_by_id(
        db: &DbConn,
        model: instruct::Model,
    ) -> anyhow::Result<instruct::Model> {
        let active_model = instruct::ActiveModel {
            id: Set(model.id),
            name: Set(model.name),
            des: Set(model.des),
            data: Set(model.data),
            ..Default::default()
        };
        SeaRepo::update_with_default::<instruct::Entity>(db, active_model).await
    }

    pub async fn insert_instruct_one(db: &DbConn, data: instruct::Model) -> anyhow::Result<String> {
        SeaRepo::insert_with_default::<instruct::Entity, _>(db, data).await
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
        ces: Option<Vec<ConditionExpression>>,
    ) -> anyhow::Result<(u64, Vec<instruct::Model>)> {
        SeaRepo::page_with_default::<instruct::Entity>(db, pg, ces).await
    }
}
