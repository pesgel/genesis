use axum::{
    extract::{Path, State},
    Json,
};
use genesis_common::{SshTargetPasswordAuth, TargetSSHOptions};
use sea_orm::sea_query::ConditionExpression;
use sea_orm::{ColumnTrait, Condition};
use tracing::error;
use uuid::Uuid;

use crate::adapter::cmd::instruct::{InstructExecuteCmd, InstructSaveCmd};
use crate::adapter::query::instruct::InstructListQuery;
use crate::adapter::vo::instruct::InstructVO;
use crate::adapter::{ExecuteReplaceItem, ResList, Response, ResponseSuccess};
use crate::common::TaskStatusEnum;
use crate::config::EXECUTE_MAP_MANAGER;
use crate::repo::model;
use crate::repo::model::instruct;
use crate::repo::sea::{ExecuteRepo, InstructRepo, NodeRepo};
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
    };
    // step4. insert execute data
    let execute_uniq_id = Uuid::new_v4().to_string();
    let mut model = model::execute::Model::new();
    model.id = execute_uniq_id.clone();
    model.name = data.name.clone();
    model.state = TaskStatusEnum::Init as i32;
    model.instruct_id = data.id;
    model.instruct_name = ins.name;
    model.node_id = data.node;
    model.node_name = node.name;
    let uuid = ExecuteRepo::insert_execute_one(&state.conn, model).await?;
    // step4. execute
    let mut pm = ProcessManger::new(execute_uniq_id.clone(), execute);
    let abort_sc = pm.get_abort_sc();
    tokio::spawn(async move {
        let mut status = TaskStatusEnum::Success;
        let mut remark = String::new();
        match pm.run(Uuid::new_v4(), option).await {
            Ok(_) => {}
            Err(e) => {
                status = TaskStatusEnum::ManualStop;
                remark = e.to_string();
            }
        }
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
    update_model.state = TaskStatusEnum::Error as i32;
    update_model.remark = e;
    let res = ExecuteRepo::update_execute_state(&state.conn, update_model).await;
    match res {
        Ok(_) => Ok(Json(ResponseSuccess::default())),
        Err(e) => Err(AppError::MsgError(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use genesis_process::InData;

    #[test]
    fn test() {
        let json = r#"{"nodes":[{"id":"1","core":{"des":"打印地址","cmd":"pwd"}},{"id":"2","pre":{"list":[{"value":"/home/yangping","match_type":"contains"}]},"core":{"des":"退出","cmd":"exit"}}],"edges":[{"source":"1","target":"2"}]}"#;
        let parsed: Result<InData, _> = serde_json::from_str(json);
        match parsed {
            Ok(data) => println!("Deserialized successfully: {:?}", data),
            Err(e) => eprintln!("Deserialization failed: {:?}", e),
        }
    }
}
