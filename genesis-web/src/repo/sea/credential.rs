//! credential repo

use crate::repo::model::credential;
use crate::repo::sea::SeaRepo;
use sea_orm::sea_query::ConditionExpression;
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, DbConn, DbErr, EntityTrait, Order, QueryFilter, QueryOrder};

pub struct CredentialRepo;

impl CredentialRepo {
    pub async fn save_credential(db: &DbConn, model: credential::Model) -> anyhow::Result<String> {
        if model.id.is_empty() {
            CredentialRepo::insert_credential_one(db, model).await
        } else {
            CredentialRepo::update_credential_by_id(db, model)
                .await
                .map(|data| anyhow::Ok(data.id))?
        }
    }
    pub async fn update_credential_by_id(
        db: &DbConn,
        model: credential::Model,
    ) -> anyhow::Result<credential::Model> {
        let mut active_model = credential::ActiveModel {
            id: Set(model.id),
            ..Default::default()
        };
        if model.status > 0 {
            active_model.status = Set(model.status)
        }
        if !model.remark.is_empty() {
            active_model.remark = Set(model.remark);
        }
        if !model.asset_id.is_empty() {
            active_model.asset_id = Set(model.asset_id);
        }
        if !model.principal.is_empty() {
            active_model.principal = Set(model.principal);
        }
        if !model.credential.is_empty() {
            active_model.credential = Set(model.credential);
        }
        if !model.protocol_id.is_empty() {
            active_model.protocol_id = Set(model.protocol_id);
        }
        SeaRepo::update_with_default::<credential::Entity>(db, active_model).await
    }

    pub async fn insert_credential_one(
        db: &DbConn,
        data: credential::Model,
    ) -> anyhow::Result<String> {
        SeaRepo::insert_with_default::<credential::Entity, _>(db, data).await
    }
    pub async fn get_credential_by_id(db: &DbConn, id: &str) -> Result<credential::Model, DbErr> {
        credential::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("not found".to_string()))
    }

    pub async fn find_credential_by(
        db: &DbConn,
        pg: (u64, u64),
        search: Option<Vec<ConditionExpression>>,
    ) -> anyhow::Result<(u64, Vec<credential::Model>)> {
        SeaRepo::page_with_default::<credential::Entity>(db, pg, search).await
    }

    pub async fn credential_select_kv(db: &DbConn) -> Result<Vec<credential::Model>, DbErr> {
        credential::Entity::find()
            .filter(credential::Column::Deleted.eq(0))
            .order_by(credential::Column::CreatedAt, Order::Desc)
            .all(db)
            .await
    }
}
