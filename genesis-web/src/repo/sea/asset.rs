//! asset repo

use crate::repo::model::asset;
use crate::repo::sea::SeaRepo;
use sea_orm::sea_query::ConditionExpression;
use sea_orm::{
    ColumnTrait, DbConn, DbErr, EntityTrait, IntoActiveModel, Order, QueryFilter, QueryOrder,
};

pub struct AssetRepo;

impl AssetRepo {
    pub async fn save_asset(db: &DbConn, model: asset::Model) -> anyhow::Result<String> {
        if model.id.is_empty() {
            AssetRepo::insert_asset_one(db, model).await
        } else {
            AssetRepo::update_asset_by_id(db, model)
                .await
                .map(|data| anyhow::Ok(data.id))?
        }
    }
    pub async fn update_asset_by_id(
        db: &DbConn,
        model: asset::Model,
    ) -> anyhow::Result<asset::Model> {
        let active_model = model.into_active_model();
        // if model.status > 0 {
        //     active_model.status = Set(model.status)
        // }
        // if !model.location.is_empty() {
        //     active_model.location = Set(model.location);
        // }
        SeaRepo::update_with_default::<asset::Entity>(db, active_model).await
    }

    pub async fn insert_asset_one(db: &DbConn, data: asset::Model) -> anyhow::Result<String> {
        SeaRepo::insert_with_default::<asset::Entity, _>(db, data).await
    }
    pub async fn get_asset_by_id(db: &DbConn, id: &str) -> Result<asset::Model, DbErr> {
        asset::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("not found".to_string()))
    }

    pub async fn find_asset_by(
        db: &DbConn,
        pg: (u64, u64),
        search: Option<Vec<ConditionExpression>>,
    ) -> anyhow::Result<(u64, Vec<asset::Model>)> {
        SeaRepo::page_with_default::<asset::Entity>(db, pg, search).await
    }

    pub async fn asset_select_kv(db: &DbConn) -> Result<Vec<asset::Model>, DbErr> {
        asset::Entity::find()
            .filter(asset::Column::Deleted.eq(0))
            .order_by(asset::Column::CreatedAt, Order::Desc)
            .all(db)
            .await
    }
}
