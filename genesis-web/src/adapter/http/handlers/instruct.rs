use axum::{
    extract::{Path, State},
    Json,
};
use genesis_common::{SshTargetPasswordAuth, TargetSSHOptions};
use uuid::Uuid;

use crate::adapter::cmd::instruct::InstructSaveCmd;
use crate::adapter::query::instruct::InstructListQuery;
use crate::adapter::vo::instruct::InstructVO;
use crate::adapter::{ResList, Response, ResponseSuccess};
use crate::repo::model::instruct;
use crate::repo::sea::InstructRepo;
use crate::{
    config::AppState,
    error::{AppError, AppJson},
};
use genesis_process::{Graph, ProcessManger};

pub async fn save_instruct(
    State(state): State<AppState>,
    AppJson(data): AppJson<InstructSaveCmd>,
) -> Result<Json<ResponseSuccess>, AppError> {
    let str = serde_json::to_string(&data.data)?;
    let mut model = instruct::Model::new();
    model.data = str;
    model.name = data.name;
    if let Some(des) = data.des {
        model.des = des;
    }
    // 判断新增,还是编辑
    let mut is_add = false;
    // 设置主键
    model.id = data
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
        InstructRepo::insert_instruct_one(&state.conn, model.into()).await?;
    } else {
        InstructRepo::update_instruct_by_id(&state.conn, model).await?;
    }
    Ok(Json(ResponseSuccess::default()))
}
pub async fn execute_instruct_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ResponseSuccess>, AppError> {
    let en = InstructRepo::get_instruct_by_id(&state.conn, &id).await?;
    let data: InstructSaveCmd = serde_json::from_str(&en.data)?;
    // 构建图
    let mut graph = Graph::new();
    graph.build_from_edges(data.data).await;
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
    Ok(Json(ResponseSuccess::default()))
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
    InstructRepo::find_instruct_by(&state.conn, query.page_query.init())
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
