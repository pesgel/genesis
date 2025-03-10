use crate::adapter::cmd::node::NodeSaveCmd;
use crate::adapter::query::node::NodeListQuery;
use crate::adapter::vo::node::{NodeListItemVO, NodeVO};
use crate::adapter::vo::BaseKV;
use crate::adapter::{ResList, Response, ResponseSuccess};
use crate::config::AppState;
use crate::error::{AppError, AppJson};
use crate::repo::model::node;
use crate::repo::sea::{NodeRepo, SeaRepo};
use axum::extract::{Path, State};
use axum::Json;
use sea_orm::sea_query::ConditionExpression;
use sea_orm::{ColumnTrait, Condition};

pub async fn save_node(
    State(state): State<AppState>,
    AppJson(param): AppJson<NodeSaveCmd>,
) -> Result<Json<Response<String>>, AppError> {
    let mut model = node::Model::new();
    model.name = param.name;
    model.host = param.host;
    model.port = param.port;
    model.account = param.account;
    model.password = param.password;
    if let Some(id) = param.id {
        model.id = id;
    }
    NodeRepo::save_node(&state.conn, model)
        .await
        .map(|id| Ok(Json(Response::new_success(id))))?
}

pub async fn get_node_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<NodeVO>, AppError> {
    NodeRepo::get_node_by_id(&state.conn, &id).await.map(|d| {
        Ok(Json(NodeVO {
            id,
            name: d.name,
            host: d.host,
            account: d.account,
            port: d.port,
        }))
    })?
}
pub async fn list_node(
    State(state): State<AppState>,
    Json(query): Json<NodeListQuery>,
) -> Result<Json<Response<ResList<NodeListItemVO>>>, AppError> {
    let mut search_option = Vec::new();
    if let Some(name) = query.name {
        if !name.is_empty() {
            search_option.push(ConditionExpression::Condition(
                Condition::all().add(node::Column::Name.contains(name)),
            ))
        }
    }
    NodeRepo::find_node_by(&state.conn, query.page_query.init(), Some(search_option))
        .await
        .map(|list| {
            Ok(Json(Response::new_success(ResList::new(
                list.0,
                list.1
                    .into_iter()
                    .map(|d| NodeListItemVO {
                        id: d.id,
                        name: d.name,
                        host: d.host,
                        account: d.account,
                        port: d.port,
                        created_by: d.created_by,
                        updated_by: d.updated_by,
                        created_at: d.created_at,
                        updated_at: d.updated_at,
                    })
                    .collect(),
            ))))
        })?
}
// BaseKV

pub async fn node_select_kv_item(
    State(state): State<AppState>,
) -> Result<Json<Response<Vec<BaseKV>>>, AppError> {
    NodeRepo::node_select_kv(&state.conn).await.map(|list| {
        Ok(Json(Response::new_success(
            list.into_iter()
                .map(|d| BaseKV {
                    key: d.id,
                    value: d.name,
                })
                .collect(),
        )))
    })?
}

pub async fn delete_node_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ResponseSuccess>, AppError> {
    SeaRepo::delete_by_id::<node::Entity>(&state.conn, &id)
        .await
        .map(|_| Ok(Json(ResponseSuccess::default())))?
}
