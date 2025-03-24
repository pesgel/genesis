use std::{collections::HashMap, sync::Arc};

use crate::common::em::PreMatchTypeEnum;
use futures_util::future::BoxFuture;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub value: String,
    pub match_type: PreMatchTypeEnum,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pre {
    pub list: Vec<Item>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub list: Vec<Item>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Core {
    pub des: String,
    pub cmd: String,
    pub expire: u64,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Position {
    x: f64,
    y: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub pre: Option<Pre>,
    pub core: Core,
    pub post: Option<Post>,
    pub position: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct InData {
    #[validate(length(min = 1, message = "nodes is empty"))]
    pub nodes: Vec<Node>,
    #[validate(length(min = 1, message = "edges is empty"))]
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone)]
pub struct Execute {
    pub node: Node,
    pub children: Vec<Arc<Mutex<Execute>>>, // 使用 Mutex 使节点可变
}

#[derive(Debug)]
pub struct Graph {
    pub nodes: Option<Arc<Mutex<Execute>>>, // 用 Arc<Mutex> 包裹 Execute 节点，便于共享和修改
}

impl Graph {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Graph { nodes: None }
    }
    pub async fn start_node(self) -> Result<Arc<Mutex<Execute>>, String> {
        self.nodes.ok_or("not match root node".to_string())
    }
    // 根据 edges 构建图
    pub async fn build_from_edges(&mut self, in_data: InData) {
        let mut node_map: HashMap<String, Arc<Mutex<Execute>>> = HashMap::new();

        // 创建所有节点，并将它们放入 node_map，保持节点之间的引用关系
        for node in in_data.nodes {
            node_map.insert(
                node.id.clone(),
                Arc::new(Mutex::new(Execute {
                    node,
                    children: Vec::new(),
                })),
            );
        }
        // 建立父子关系
        for edge in in_data.edges {
            if let Some(parent_node) = node_map.get(&edge.source) {
                if let Some(child_node) = node_map.get(&edge.target) {
                    parent_node
                        .lock()
                        .await
                        .children
                        .push(Arc::clone(child_node));
                }
            }
        }
        // 将所有节点收集到 nodes 中
        self.nodes = node_map
            .into_iter()
            .filter(|(id, _)| id == "1")
            .map(|(_, node)| Arc::clone(&node))
            .next();
    }

    // 打印图的结构，递归遍历每个节点及其子节点
    pub async fn print_graph(&self) {
        let root = self.nodes.as_ref().unwrap();
        self.print_node(root, 0).await;
    }

    // 递归打印节点及其子节点
    #[allow(clippy::only_used_in_recursion)]
    pub fn print_node<'a>(
        &'a self,
        node: &'a Arc<Mutex<Execute>>,
        level: usize,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let node_ref = node.lock().await;
            // 打印当前节点信息
            println!(
                "{}Node ID: {}, Description: {} Cmd: {}",
                "  ".repeat(level),
                node_ref.node.id,
                node_ref.node.core.des,
                node_ref.node.core.cmd
            );
            // 递归打印子节点
            for child in &node_ref.children {
                self.print_node(child, level + 1).await;
            }
        })
    }
}
