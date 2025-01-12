use crate::adapter::query::execute::ExecuteListQuery;
use crate::adapter::vo::execute::{ExecuteListItemVO, ExecuteVO};
use crate::adapter::{ResList, Response};
use crate::config::AppState;
use crate::error::AppError;
use crate::repo::sea::ExecuteRepo;
use axum::extract::{Path, State};
use axum::Json;

pub async fn get_execute_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ExecuteVO>, AppError> {
    ExecuteRepo::get_execute_by_id(&state.conn, &id)
        .await
        .map(|d| {
            Ok(Json(ExecuteVO {
                id,
                name: d.name,
                state: d.state,
                remark: d.remark,
                node_id: d.node_id,
                node_name: d.node_name,
                instruct_id: d.instruct_id,
                instruct_name: d.instruct_name,
            }))
        })?
}
pub async fn list_execute(
    State(state): State<AppState>,
    Json(query): Json<ExecuteListQuery>,
) -> Result<Json<Response<ResList<ExecuteListItemVO>>>, AppError> {
    ExecuteRepo::find_execute_by(&state.conn, query.page_query.init())
        .await
        .map(|list| {
            Ok(Json(Response::new_success(ResList::new(
                list.0,
                list.1
                    .into_iter()
                    .map(|d| ExecuteListItemVO {
                        id: d.id,
                        name: d.name,
                        state: d.state,
                        remark: d.remark,
                        node_id: d.node_id,
                        node_name: d.node_name,
                        instruct_id: d.instruct_id,
                        instruct_name: d.instruct_name,
                        created_by: d.created_by,
                        updated_by: d.updated_by,
                        created_at: d.created_at,
                        updated_at: d.updated_at,
                    })
                    .collect(),
            ))))
        })?
}
