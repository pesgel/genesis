use axum::{
    extract::{Path, State},
    Json,
};
use genesis_common::{SshTargetPasswordAuth, TargetSSHOptions};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    adapter::cqe::Response,
    config::AppState,
    error::{AppError, AppJson},
};
use genesis_process::{Graph, InData, ProcessManger};

pub async fn get_instruct_by_id(Path(_): Path<String>) -> Result<Json<Value>, AppError> {
    // 使用 `serde_json::json!` 构建 JSON 数据
    let response_data = json!(
            {
        "nodes": [{
            "id": "1",
            "core": {
                "des": "修改密码",
                "cmd": "passwd"
            },
            "position": {
                "x": 85.08970203951566,
                "y": 13.90948773103888
            }
        }, {
            "id": "2",
            "pre": {
                "list": [{
                    "value": "current",
                    "matchType": "contains"
                }]
            },
            "core": {
                "des": "输入当前密码",
                "cmd": "%]m73MmQ"
            },
            "position": {
                "x": 85.88779397705542,
                "y": 84.05067957297638
            }
        }, {
            "id": "3",
            "pre": {
                "list": [{
                    "value": "New password",
                    "matchType": "contains"
                }]
            },
            "core": {
                "des": "输入新密码",
                "cmd": "#wR61V(s"
            },
            "position": {
                "x": -21.5,
                "y": 176.5
            }
        }, {
            "id": "4",
            "pre": {
                "list": [{
                    "value": "Retype new password",
                    "matchType": "contains"
                }]
            },
            "core": {
                "des": "确认新密码",
                "cmd": "#wR61V(s"
            },
            "position": {
                "x": -20.541787019541346,
                "y": 245.11686020076843
            }
        }, {
            "id": "5",
            "core": {
                "des": "退出",
                "cmd": "exit"
            },
            "position": {
                "x": 76.13174833561897,
                "y": 400.2398151274464
            }
        }, {
            "id": "6",
            "pre": {
                "list": [{
                    "value": "current",
                    "matchType": "contains"
                }]
            },
            "core": {
                "des": "密码错误",
                "cmd": "#wR61V(s"
            },
            "position": {
                "x": 189.84280866357432,
                "y": 174.3101668743471
            }
        }, {
            "id": "7",
            "pre": {
                "list": [{
                    "value": "New password\t",
                    "matchType": "contains"
                }]
            },
            "core": {
                "des": "新密码",
                "cmd": "%]m73MmQ"
            },
            "position": {
                "x": 193.77941241654605,
                "y": 244.1503166263235
            }
        }, {
            "id": "8",
            "pre": {
                "list": [{
                    "value": "Retype new password\t",
                    "matchType": "contains"
                }]
            },
            "core": {
                "des": "确认新密码",
                "cmd": "%]m73MmQ"
            },
            "position": {
                "x": 192.87941048675248,
                "y": 319.9304791149371
            }
        }],
        "edges": [{
            "source": "1",
            "target": "2"
        }, {
            "source": "2",
            "target": "3"
        }, {
            "source": "3",
            "target": "4"
        }, {
            "source": "4",
            "target": "5"
        }, {
            "source": "8",
            "target": "5"
        }, {
            "source": "6",
            "target": "7"
        }, {
            "source": "7",
            "target": "8"
        }, {
            "source": "2",
            "target": "6"
        }]
    }
        );
    // 返回 JSON 数据
    Ok(Json(response_data))
}

pub async fn save_and_execute(
    State(_): State<AppState>,
    AppJson(data): AppJson<InData>,
) -> Result<Json<Response>, AppError> {
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
        host: "106.54.225.80".into(),
        port: 22,
        username: "yangping".into(),
        allow_insecure_algos: Some(true),
        auth: genesis_common::SSHTargetAuth::Password(SshTargetPasswordAuth {
            password: "#wR61V(s".to_string(),
        }),
    };
    // 执行
    pm.run(Uuid::new_v4(), option).await?;
    Ok(Json(Response::default()))
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
