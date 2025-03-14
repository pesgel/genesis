use crate::adapter::query::execute::ExecuteListQuery;
use crate::adapter::vo::execute::{ExecuteListItemVO, ExecuteVO};
use crate::adapter::{ResList, Response, ResponseSuccess};
use crate::config::{AppState, SHARED_APP_CONFIG};
use crate::error::AppError;
use crate::repo::model::execute;
use crate::repo::sea::{ExecuteRepo, SeaRepo};
use axum::extract::{Path, State};
use axum::http;
use axum::response::IntoResponse;
use axum::Json;
use sea_orm::sea_query::ConditionExpression;
use sea_orm::{ColumnTrait, Condition};

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
    let mut search_option = Vec::new();
    if let Some(name) = query.name {
        if !name.is_empty() {
            search_option.push(ConditionExpression::Condition(
                Condition::all().add(execute::Column::Name.contains(name)),
            ))
        }
    }
    ExecuteRepo::find_execute_by(&state.conn, query.page_query.init(), Some(search_option))
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

pub async fn delete_execute_history_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ResponseSuccess>, AppError> {
    SeaRepo::delete_by_id::<execute::Entity>(&state.conn, &id)
        .await
        .map(|_| Ok(Json(ResponseSuccess::default())))?
}

pub async fn execute_recording(Path(id): Path<String>) -> impl IntoResponse {
    let base_path = &SHARED_APP_CONFIG.read().await.server.recording_path;
    let file_path = std::path::Path::new(base_path)
        .join("ssh")
        .join(id)
        .join("recording.cast");
    match file_path.canonicalize() {
        Ok(real_path) => {
            if !real_path.starts_with(base_path) {
                return http::StatusCode::FORBIDDEN.into_response();
            }
            match tokio::fs::File::open(&real_path).await {
                Ok(file) => {
                    let stream = tokio_util::io::ReaderStream::new(file);
                    // 设置 `Content-Disposition` 让浏览器下载文件
                    http::Response::builder()
                        .status(http::StatusCode::OK)
                        .header(http::header::CONTENT_TYPE, "application/octet-stream")
                        .header(
                            http::header::CONTENT_DISPOSITION,
                            format!("attachment; filename=\"{}\"", "recording.cast"),
                        )
                        //.body(axum::body::Body::from(buffer))
                        .body(axum::body::Body::from_stream(stream))
                        .unwrap()
                        .into_response()
                }
                Err(_) => http::StatusCode::BAD_REQUEST.into_response(),
            }
        }
        Err(_) => http::StatusCode::BAD_REQUEST.into_response(),
    }
}
