//! user repo
use crate::repo::model::user;
use crate::repo::sea::SeaRepo;
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, DbConn, DbErr, EntityTrait, QueryFilter};

pub struct UserRepo;

impl UserRepo {
    pub async fn find_user_by_username(db: &DbConn, username: &str) -> Result<user::Model, DbErr> {
        user::Entity::find()
            .filter(user::Column::Username.eq(username))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("not found".to_string()))
    }
    pub async fn update_user_by_id(db: &DbConn, model: user::Model) -> anyhow::Result<user::Model> {
        let active_model = user::ActiveModel {
            id: Set(model.id),
            name: Set(model.name),
            remark: Set(model.remark),
            password: Set(model.password),
            username: Set(model.username),
            phone: Set(model.phone),
            ..Default::default()
        };
        SeaRepo::update_with_default::<user::Entity>(db, active_model).await
    }

    pub async fn insert_user_one(db: &DbConn, data: user::Model) -> anyhow::Result<String> {
        SeaRepo::insert_with_default::<user::Entity, _>(db, data).await
    }
    pub async fn get_user_by_id(db: &DbConn, id: &str) -> Result<user::Model, DbErr> {
        user::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("not found".to_string()))
    }

    pub async fn find_user_by(
        db: &DbConn,
        pg: (u64, u64),
    ) -> anyhow::Result<(u64, Vec<user::Model>)> {
        SeaRepo::page_with_default::<user::Entity>(db, pg, None).await
    }
}
