//! node repo
use crate::repo::model::node;
use crate::repo::sea::SeaRepo;
use sea_orm::sea_query::ConditionExpression;
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, DbConn, DbErr, EntityTrait, Order, QueryFilter, QueryOrder};

pub struct NodeRepo;

impl NodeRepo {
    pub async fn save_node(db: &DbConn, model: node::Model) -> anyhow::Result<String> {
        if model.id.is_empty() {
            NodeRepo::insert_node_one(db, model).await
        } else {
            NodeRepo::update_node_by_id(db, model)
                .await
                .map(|data| anyhow::Ok(data.id))?
        }
    }
    pub async fn update_node_by_id(db: &DbConn, model: node::Model) -> anyhow::Result<node::Model> {
        let active_model = node::ActiveModel {
            id: Set(model.id),
            name: Set(model.name),
            host: Set(model.host),
            password: Set(model.password),
            port: Set(model.port),
            account: Set(model.account),
            remark: Set(model.remark),
            ..Default::default()
        };
        SeaRepo::update_with_default::<node::Entity>(db, active_model).await
    }

    pub async fn insert_node_one(db: &DbConn, data: node::Model) -> anyhow::Result<String> {
        SeaRepo::insert_with_default::<node::Entity, _>(db, data).await
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
        search: Option<Vec<ConditionExpression>>,
    ) -> anyhow::Result<(u64, Vec<node::Model>)> {
        SeaRepo::page_with_default::<node::Entity>(db, pg, search).await
    }

    pub async fn node_select_kv(db: &DbConn) -> Result<Vec<node::Model>, DbErr> {
        node::Entity::find()
            .filter(node::Column::Deleted.eq(0))
            .order_by(node::Column::CreatedAt, Order::Desc)
            .all(db)
            .await
    }
}
