use crate::adapter::cmd::asset::AssetSaveCmd;
use crate::adapter::query::asset::{AssetListQuery, AssetProtocolListQuery};
use crate::adapter::vo::asset::{AssetListItemVO, AssetProtocolListItemVO, AssetVO};
use crate::adapter::{ResList, Response, ResponseSuccess};
use crate::config::AppState;
use crate::error::{AppError, AppJson};
use crate::repo::model::{asset, protocol};
use crate::repo::sea;
use crate::repo::sea::{AssetRepo, ProtocolRepo, SeaRepo};
use axum::extract::{Path, State};
use axum::Json;
use sea_orm::sea_query::ConditionExpression;
use sea_orm::{ColumnTrait, Condition};
use serde_json::json;

pub async fn save_asset(
    State(state): State<AppState>,
    AppJson(param): AppJson<AssetSaveCmd>,
) -> Result<Json<ResponseSuccess>, AppError> {
    let model = genesis_common::copy::<_, asset::Model>(&param)?;
    // step1. save protocol
    let id = AssetRepo::save_asset(&state.conn, model).await?;
    if let Some(pts) = param.protocol_list {
        ProtocolRepo::save_batch_protocol(
            &state.conn,
            pts.iter()
                .map(|d| {
                    let mut model = protocol::Model::new();
                    model.asset_id = id.clone();
                    model.port = d.port;
                    model.protocol = d.protocol.clone();
                    model
                })
                .collect(),
        )
        .await?;
    }
    Ok(Json(ResponseSuccess::default()))
}

pub async fn get_asset_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AssetVO>, AppError> {
    AssetRepo::get_asset_by_id(&state.conn, &id)
        .await
        .map(|d| Ok(Json(genesis_common::copy(&d).unwrap_or(AssetVO::default()))))?
}
pub async fn list_asset(
    State(state): State<AppState>,
    Json(query): Json<AssetListQuery>,
) -> Result<Json<Response<ResList<AssetListItemVO>>>, AppError> {
    let mut search_option = Vec::new();
    if let Some(name) = query.name {
        if !name.is_empty() {
            search_option.push(ConditionExpression::Condition(
                Condition::all().add(asset::Column::Name.contains(name)),
            ))
        }
    }
    if let Some(name) = query.alias_name {
        if !name.is_empty() {
            search_option.push(ConditionExpression::Condition(
                Condition::all().add(asset::Column::AliasName.contains(name)),
            ))
        }
    }

    if let Some(name) = query.address {
        if !name.is_empty() {
            search_option.push(ConditionExpression::Condition(
                Condition::all().add(asset::Column::Address.contains(name)),
            ))
        }
    }

    if let Some(ty) = query.asset_type {
        search_option.push(ConditionExpression::Condition(
            Condition::all().add(asset::Column::AssetType.eq(ty)),
        ))
    }
    if let Some(ty) = query.address_type {
        search_option.push(ConditionExpression::Condition(
            Condition::all().add(asset::Column::AddressType.eq(ty)),
        ))
    }

    AssetRepo::find_asset_by(&state.conn, query.page_query.init(), Some(search_option))
        .await
        .map(|list| {
            Ok(Json(Response::new_success(ResList::new(
                list.0,
                list.1
                    .into_iter()
                    .map(|d| genesis_common::copy(&d).unwrap_or(AssetListItemVO::default()))
                    .collect(),
            ))))
        })?
}

pub async fn delete_asset_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ResponseSuccess>, AppError> {
    // TODO 关联校验
    SeaRepo::delete_by_id::<asset::Entity>(&state.conn, &id).await?;
    // 删除关联协议数据
    SeaRepo::update_with_map::<protocol::Entity>(
        &state.conn,
        json!({
            "deleted": 1
        }),
        json!({
            "asset_id": id
        }),
    )
    .await
    .map(|_| Ok(Json(ResponseSuccess::default())))?
}

pub async fn list_asset_protocol(
    State(state): State<AppState>,
    Json(query): Json<AssetProtocolListQuery>,
) -> Result<Json<Response<ResList<AssetProtocolListItemVO>>>, AppError> {
    let (sql, values) = sea::SqlBuilder::new(
        r#"
SELECT
	a.id AS asset_id,
	a.NAME AS asset_name,
	a.address AS address,
	ap.protocol AS protocol,
	ap.id AS protocol_id,
	ap.port AS port
FROM
	asset a
	LEFT JOIN asset_protocol ap ON a.id = ap.asset_id
WHERE
	a.deleted = 0
	AND ap.deleted =0
    "#,
    )
    .try_filter_like("a.name", query.asset_name)
    .build();
    let res = SeaRepo::page_with_sql::<AssetProtocolListItemVO>(
        &state.conn,
        query.page_query.init(),
        sql,
        values,
        query.page_query.order_clause(),
    )
    .await?;
    Ok(Json(Response::new_success(ResList::new(res.0, res.1))))
}
