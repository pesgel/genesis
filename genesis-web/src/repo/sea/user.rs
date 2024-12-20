//! user repo
use crate::repo::model::user;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, PaginatorTrait, QueryFilter,
};
use tracing::info;

pub struct UserRepo;

impl UserRepo {
    pub async fn find_user_by_username(db: &DbConn, username: &str) -> Result<user::Model, DbErr> {
        user::Entity::find()
            .filter(user::Column::Username.eq(username))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("not found".to_string()))
    }
    pub async fn update_user_by_id(db: &DbConn, model: user::Model) -> Result<user::Model, DbErr> {
        user::ActiveModel {
            id: Set(model.id),
            name: Set(model.name),
            remark: Set(model.remark),
            password: Set(model.password),
            username: Set(model.username),
            phone: Set(model.phone),
            ..Default::default()
        }
        .update(db)
        .await
    }

    pub async fn insert_user_one(db: &DbConn, data: user::ActiveModel) -> Result<(), DbErr> {
        let res = user::Entity::insert(data).exec(db).await;
        match res {
            Ok(_) => Ok(()),
            Err(DbErr::RecordNotInserted) => Ok(()),
            Err(e) => Err(e),
        }
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
    ) -> Result<(u64, Vec<user::Model>), DbErr> {
        info!("{:?}", pg);
        let ens = user::Entity::find().paginate(db, pg.1);
        let count = ens.num_items().await?;
        let res = ens.fetch_page(0).await?;
        Ok((count, res))
    }
}
