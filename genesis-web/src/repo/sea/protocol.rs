//! protocol repo

use crate::repo::model::protocol;
use crate::repo::sea::SeaRepo;
use sea_orm::sea_query::ConditionExpression;
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, DbConn, DbErr, EntityTrait, Order, QueryFilter, QueryOrder};

pub struct ProtocolRepo;

impl ProtocolRepo {
    pub async fn save_batch_protocol(
        db: &DbConn,
        data: Vec<protocol::Model>,
    ) -> anyhow::Result<Vec<String>> {
        SeaRepo::insert_many_with_default::<protocol::Entity, _>(db, data).await
    }
    pub async fn save_protocol(db: &DbConn, model: protocol::Model) -> anyhow::Result<String> {
        if model.id.is_empty() {
            ProtocolRepo::insert_protocol_one(db, model).await
        } else {
            ProtocolRepo::update_protocol_by_id(db, model)
                .await
                .map(|data| anyhow::Ok(data.id))?
        }
    }
    pub async fn update_protocol_by_id(
        db: &DbConn,
        model: protocol::Model,
    ) -> anyhow::Result<protocol::Model> {
        let mut active_model = protocol::ActiveModel {
            id: Set(model.id),
            remark: Set(model.remark),
            ..Default::default()
        };
        if model.status > 0 {
            active_model.status = Set(model.status)
        }
        SeaRepo::update_with_default::<protocol::Entity>(db, active_model).await
    }

    pub async fn insert_protocol_one(db: &DbConn, data: protocol::Model) -> anyhow::Result<String> {
        SeaRepo::insert_with_default::<protocol::Entity, _>(db, data).await
    }
    pub async fn get_protocol_by_id(db: &DbConn, id: &str) -> Result<protocol::Model, DbErr> {
        protocol::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("not found".to_string()))
    }

    pub async fn find_protocol_by(
        db: &DbConn,
        pg: (u64, u64),
        search: Option<Vec<ConditionExpression>>,
    ) -> anyhow::Result<(u64, Vec<protocol::Model>)> {
        SeaRepo::page_with_default::<protocol::Entity>(db, pg, search).await
    }

    pub async fn protocol_select_kv(db: &DbConn) -> Result<Vec<protocol::Model>, DbErr> {
        protocol::Entity::find()
            .filter(protocol::Column::Deleted.eq(0))
            .order_by(protocol::Column::CreatedAt, Order::Desc)
            .all(db)
            .await
    }
}
