//! execute repo
use crate::repo::model;
use crate::repo::sea::SeaRepo;
use sea_orm::sea_query::ConditionExpression;
use sea_orm::ActiveValue::Set;
use sea_orm::{DbConn, DbErr, EntityTrait};

pub struct ExecuteRepo;

impl ExecuteRepo {
    pub async fn update_execute_state(
        db: &DbConn,
        model: model::execute::Model,
    ) -> anyhow::Result<model::execute::Model> {
        let mut active_model = model::execute::ActiveModel {
            id: Set(model.id),
            ..Default::default()
        };
        if model.state > 0 {
            active_model.state = Set(model.state)
        }
        if !model.remark.is_empty() {
            active_model.remark = Set(model.remark)
        }
        SeaRepo::update_with_default::<model::execute::Entity>(db, active_model).await
    }

    pub async fn insert_execute_one(
        db: &DbConn,
        data: model::execute::Model,
    ) -> anyhow::Result<String> {
        SeaRepo::insert_with_default::<model::execute::Entity, _>(db, data).await
    }
    pub async fn get_execute_by_id(db: &DbConn, id: &str) -> Result<model::execute::Model, DbErr> {
        model::execute::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("not found".to_string()))
    }

    pub async fn find_execute_by(
        db: &DbConn,
        pg: (u64, u64),
        ces: Option<Vec<ConditionExpression>>,
    ) -> anyhow::Result<(u64, Vec<model::execute::Model>)> {
        SeaRepo::page_with_default::<model::execute::Entity>(db, pg, ces).await
    }
}
