use axum::{
    extract::{Path, State},
    Json,
};
use genesis_common::{SshTargetPasswordAuth, TargetSSHOptions, TaskStatusEnum};
use sea_orm::sea_query::ConditionExpression;
use sea_orm::{ColumnTrait, Condition};
use tracing::error;
use uuid::Uuid;

use crate::adapter::cmd::instruct::{InstructExecuteCmd, InstructSaveCmd};
use crate::adapter::query::instruct::InstructListQuery;
use crate::adapter::vo::instruct::InstructVO;
use crate::adapter::{ExecuteReplaceItem, ResList, Response, ResponseSuccess};
use crate::config::{EXECUTE_MAP_MANAGER, SHARED_APP_CONFIG};
use crate::repo::model;
use crate::repo::model::instruct;
use crate::repo::sea::{ExecuteRepo, InstructRepo, NodeRepo, SeaRepo};
use crate::{
    config::AppState,
    error::{AppError, AppJson},
};
use genesis_process::{Graph, InData, ProcessManger};

pub async fn save_instruct(
    State(state): State<AppState>,
    AppJson(data): AppJson<InstructSaveCmd>,
) -> Result<Json<Response<String>>, AppError> {
    let str = serde_json::to_string(&data.data)?;
    let mut model = instruct::Model::new();
    model.data = str;
    model.name = data.name;
    if let Some(des) = data.des {
        model.des = des;
    }
    if let Some(id) = data.id {
        model.id = id;
    }
    InstructRepo::save_instruct(&state.conn, model)
        .await
        .map(|id| Ok(Json(Response::new_success(id))))?
}
pub async fn get_instruct_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<InstructVO>, AppError> {
    InstructRepo::get_instruct_by_id(&state.conn, &id)
        .await
        .map(|d| {
            Ok(Json(InstructVO {
                id: d.id,
                data: d.data,
                name: d.name,
                des: d.des,
                created_by: d.created_by,
                updated_by: d.updated_by,
                created_at: d.created_at,
                updated_at: d.updated_at,
            }))
        })?
}
pub async fn list_instruct(
    State(state): State<AppState>,
    Json(query): Json<InstructListQuery>,
) -> Result<Json<Response<ResList<InstructVO>>>, AppError> {
    let mut search_option = Vec::new();
    if let Some(name) = query.name {
        if !name.is_empty() {
            search_option.push(ConditionExpression::Condition(
                Condition::all().add(instruct::Column::Name.contains(name)),
            ))
        }
    }
    InstructRepo::find_instruct_by(&state.conn, query.page_query.init(), Some(search_option))
        .await
        .map(|list| {
            Ok(Json(Response::new_success(ResList::new(
                list.0,
                list.1
                    .into_iter()
                    .map(|d| InstructVO {
                        id: d.id,
                        data: d.data,
                        name: d.name,
                        des: d.des,
                        created_by: d.created_by,
                        updated_by: d.updated_by,
                        created_at: d.created_at,
                        updated_at: d.updated_at,
                    })
                    .collect(),
            ))))
        })?
}

pub async fn replace_execute_param(
    mut old_str: String,
    rep: Vec<ExecuteReplaceItem>,
) -> anyhow::Result<String> {
    rep.into_iter()
        .for_each(|item| old_str = old_str.replace(&item.mark, &item.value));
    anyhow::Ok(old_str)
}

pub async fn execute_instruct(
    State(state): State<AppState>,
    AppJson(data): AppJson<InstructExecuteCmd>,
) -> Result<Json<ResponseSuccess>, AppError> {
    // step1. fetch instruct data
    let ins = InstructRepo::get_instruct_by_id(&state.conn, &data.id).await?;
    let replaces = serde_json::to_string(&data.replaces)?;
    // replace param
    let new_str = replace_execute_param(ins.data, data.replaces).await?;
    let in_data: InData = serde_json::from_str(&new_str)?;
    // step2. build graph
    let mut graph = Graph::new();
    graph.build_from_edges(in_data).await;
    let execute = graph.start_node().await?;
    // step3. set ssh options
    let node = NodeRepo::get_node_by_id(&state.conn, &data.node).await?;
    let option = TargetSSHOptions {
        host: node.host,
        port: node.port as u16,
        username: node.account,
        allow_insecure_algos: Some(true),
        auth: genesis_common::SSHTargetAuth::Password(SshTargetPasswordAuth {
            password: node.password,
        }),
        // TODO pty param
        pty_request: Default::default(),
    };
    // step4. insert execute data
    let execute_uniq_id = Uuid::new_v4().to_string();
    let mut model = model::execute::Model::new();
    model.id = execute_uniq_id.clone();
    model.name = data.name.clone();
    model.state = genesis_common::TaskStatusEnum::Init as i32;
    model.instruct_id = data.id;
    model.instruct_name = ins.name;
    model.node_id = data.node;
    model.node_name = node.name;
    model.replaces = replaces;
    let uuid = ExecuteRepo::insert_execute_one(&state.conn, model).await?;
    // step4. execute
    let mut pm = ProcessManger::new(execute_uniq_id.clone(), execute)?.with_recorder_param(
        &SHARED_APP_CONFIG.read().await.server.recording_path,
        &option.pty_request.term,
        option.pty_request.height,
        option.pty_request.width,
    )?;
    let abort_sc = pm.get_abort_sc();
    tokio::spawn(async move {
        // = TaskStatusEnum::Success;
        let mut remark = String::new();
        let status = match pm.run(Uuid::new_v4(), option).await {
            Ok(em) => em,
            Err(e) => {
                remark = e.to_string();
                TaskStatusEnum::Error
            }
        };
        let mut update_model = model::execute::Model::new();
        update_model.id = uuid;
        update_model.state = status as i32;
        update_model.remark = remark;
        let res = ExecuteRepo::update_execute_state(&state.conn, update_model).await;
        match res {
            Ok(_) => {}
            Err(e) => {
                error!("update state error: {:?}", e)
            }
        }
    });
    // step5. register global manager
    EXECUTE_MAP_MANAGER
        .write()
        .await
        .insert(execute_uniq_id, abort_sc);
    Ok(Json(ResponseSuccess::default()))
}

/// stop execute task
pub async fn stop_execute_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ResponseSuccess>, AppError> {
    let res = match EXECUTE_MAP_MANAGER.read().await.get(&id) {
        None => {
            let _ = stop_execute_callback(
                &state,
                id.clone(),
                "execute task is not running".to_string(),
            )
            .await?;
            Err(AppError::MsgError(
                "execute task is not running".to_string(),
            ))
        }
        Some(value) => match value.send(true).map_err(|e| anyhow::anyhow!(e)) {
            Ok(_) => {
                //EXECUTE_MAP_MANAGER.write().await.remove(&id);
                Ok(Json(ResponseSuccess::default()))
            }
            Err(e) => stop_execute_callback(&state, id.clone(), e.to_string()).await,
        },
    }?;
    EXECUTE_MAP_MANAGER.write().await.remove(&id);
    Ok(res)
}

async fn stop_execute_callback(
    state: &AppState,
    id: String,
    e: String,
) -> Result<Json<ResponseSuccess>, AppError> {
    let mut update_model = model::execute::Model::new();
    update_model.id = id;
    update_model.state = TaskStatusEnum::ManualStop as i32;
    update_model.remark = e;
    let res = ExecuteRepo::update_execute_state(&state.conn, update_model).await;
    match res {
        Ok(_) => Ok(Json(ResponseSuccess::default())),
        Err(e) => Err(AppError::MsgError(e.to_string())),
    }
}

pub async fn delete_instruct_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ResponseSuccess>, AppError> {
    SeaRepo::delete_by_id::<instruct::Entity>(&state.conn, &id)
        .await
        .map(|_| Ok(Json(ResponseSuccess::default())))?
}
