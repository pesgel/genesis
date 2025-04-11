use std::sync::Arc;

use axum::async_trait;
use dashmap::DashMap;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::types::{SessionContextTrait, SessionManagerTrait, SessionTypeEnum};

#[derive(Clone, Default)]
pub struct MemorySessionManager {
    storage: DashMap<Uuid, Arc<Mutex<dyn SessionContextTrait + Send + Sync>>>,
}

#[async_trait]
impl SessionManagerTrait for MemorySessionManager {
    async fn count(&self) -> i32 {
        self.storage.len() as i32
    }

    // 从 storage 中移除指定 id 的会话
    async fn remove(&self, id: Uuid) -> anyhow::Result<()> {
        if let Some((_, v)) = self.storage.remove(&id) {
            let _ = v.lock().await.close().await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("session not found"))
        }
    }

    // 将指定会话注册到 storage 中
    async fn register(
        &self,
        id: Uuid,
        ctx: Arc<Mutex<dyn SessionContextTrait + Send + Sync>>,
    ) -> anyhow::Result<()> {
        self.storage.insert(id, ctx);
        Ok(())
    }

    // 从 storage 中查找指定 id 的会话
    async fn pick(
        &self,
        id: Uuid,
    ) -> anyhow::Result<Arc<Mutex<dyn SessionContextTrait + Send + Sync>>> {
        if let Some(session) = self.storage.get(&id) {
            Ok(session.clone())
        } else {
            Err(anyhow::anyhow!("Session not found"))
        }
    }
}

pub struct SSHSessionCtx {
    id: Uuid,
    close: bool,
    on_close: Option<Box<dyn Fn() + Send + Sync>>,
}

impl SSHSessionCtx {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            close: false,
            on_close: None,
        }
    }
    pub fn with_on_close(mut self, on_close: Option<Box<dyn Fn() + Send + Sync>>) -> Self {
        self.on_close = on_close;
        self
    }
}

#[async_trait]
impl SessionContextTrait for SSHSessionCtx {
    async fn closed(&self) -> bool {
        self.close
    }
    async fn close(&mut self) -> anyhow::Result<()> {
        self.close = true;
        if let Some(f) = self.on_close.take() {
            f()
        }
        anyhow::Ok(())
    }
    async fn get_session_id(&self) -> Uuid {
        self.id
    }
    async fn get_session_type(&self) -> SessionTypeEnum {
        SessionTypeEnum::SSH
    }
}
