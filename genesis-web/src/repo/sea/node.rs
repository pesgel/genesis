//! node repo
use crate::repo::model::node;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DbConn, DbErr, EntityTrait, PaginatorTrait};
use tracing::info;

pub struct NodeRepo;

impl NodeRepo {
    pub async fn update_node_by_id(db: &DbConn, model: node::Model) -> Result<node::Model, DbErr> {
        node::ActiveModel {
            id: Set(model.id),
            name: Set(model.name),
            host: Set(model.host),
            password: Set(model.password),
            port: Set(model.port),
            account: Set(model.account),
            ..Default::default()
        }
        .update(db)
        .await
    }

    pub async fn insert_node_one(db: &DbConn, data: node::ActiveModel) -> Result<(), DbErr> {
        let res = node::Entity::insert(data).exec(db).await;
        match res {
            Ok(_) => Ok(()),
            Err(DbErr::RecordNotInserted) => Ok(()),
            Err(e) => Err(e),
        }
    }
    pub async fn get_node_by_id(db: &DbConn, id: &str) -> Result<node::Model, DbErr> {
        node::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("not found".to_string()))
    }

    pub async fn find_node_by(
        db: &DbConn,
        pg: (u64, u64),
    ) -> Result<(u64, Vec<node::Model>), DbErr> {
        info!("{:?}", pg);
        let ens = node::Entity::find().paginate(db, pg.1);
        let count = ens.num_items().await?;
        let res = ens.fetch_page(0).await?;
        Ok((count, res))
    }
}
