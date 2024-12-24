use crate::adapter::cmd::node::NodeSaveCmd;
use crate::adapter::query::node::NodeListQuery;
use crate::adapter::vo::node::{NodeListItemVO, NodeVO};
use crate::adapter::{ResList, Response, ResponseSuccess};
use crate::config::AppState;
use crate::error::{AppError, AppJson};
use crate::repo::model::node;
use crate::repo::sea::NodeRepo;
use axum::extract::{Path, State};
use axum::Json;
use uuid::Uuid;

pub async fn save_node(
    State(state): State<AppState>,
    AppJson(param): AppJson<NodeSaveCmd>,
) -> Result<Json<ResponseSuccess>, AppError> {
    let mut model = node::Model::new();
    model.name = param.name;
    model.host = param.host;
    model.port = param.port;
    model.account = param.account;
    model.password = param.password;
    // 判断新增,还是编辑
    let mut is_add = false;
    // 设置主键
    model.id = param
        .id
        .filter(|e| {
            if e.is_empty() {
                is_add = true;
                false
            } else {
                true
            }
        })
        .unwrap_or_else(|| {
            is_add = true;
            Uuid::new_v4().to_string()
        });
    if is_add {
        NodeRepo::insert_node_one(&state.conn, model.into()).await?;
    } else {
        NodeRepo::update_node_by_id(&state.conn, model).await?;
    }
    Ok(Json(ResponseSuccess::default()))
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
    NodeRepo::find_node_by(&state.conn, query.page_query.init())
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
