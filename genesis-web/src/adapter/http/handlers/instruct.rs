use axum::{
    extract::{Path, State},
    Json,
};
use genesis_common::{SshTargetPasswordAuth, TargetSSHOptions};
use tracing::error;
use uuid::Uuid;

use crate::adapter::vo::instruct::InstructVO;
use crate::repo::model::instruct;
use crate::repo::sea::InstructRepo;
use crate::{
    adapter::cqe::Response,
    config::AppState,
    error::{AppError, AppJson},
};
use genesis_process::{Graph, InData, ProcessManger};

pub async fn save_and_execute(
    State(state): State<AppState>,
    AppJson(data): AppJson<InData>,
) -> Result<Json<Response>, AppError> {
    match serde_json::to_string(&data) {
        Ok(str) => {
            let mut model = instruct::Model::new();
            model.id = Uuid::new_v4().to_string();
            model.name = "test".to_string();
            model.data = str;
            InstructRepo::insert_instruct_one(&state.conn, model.into()).await?;
        }
        Err(e) => {
            error!("marshal in data error:{:?}", e)
        }
    }
    // 构建图
    let mut graph = Graph::new();
    graph.build_from_edges(data).await;
    // 打印图的结构
    graph.print_graph().await;
    let execute = graph.start_node().await.unwrap();
    let mut pm = ProcessManger { execute };
    // TODO 获取数据库数据
    // let new_password = "%]m73MmQ";
    // let old_password = "#wR61V(s";
    // TODO 敏感信息排除
    let option = TargetSSHOptions {
        host: "127.0.0.1".into(),
        port: 32222,
        username: "root".into(),
        allow_insecure_algos: Some(true),
        auth: genesis_common::SSHTargetAuth::Password(SshTargetPasswordAuth {
            password: "123456".to_string(),
        }),
    };
    // 执行
    pm.run(Uuid::new_v4(), option).await?;
    Ok(Json(Response::default()))
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
) -> Result<Json<Vec<InstructVO>>, AppError> {
    InstructRepo::find_instruct_by(&state.conn)
        .await
        .map(|list| {
            Ok(Json(
                list.into_iter()
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
            ))
        })?
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
