use std::sync::Arc;

use axum::async_trait;
use strum::{AsRefStr, FromRepr};
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, FromRepr, AsRefStr)]
pub enum SessionTypeEnum {
    SSH,
}

#[async_trait]
pub trait SessionManagerTrait {
    async fn count(&self) -> i32;
    async fn remove(&self, id: Uuid) -> anyhow::Result<()>;
    async fn register(
        &self,
        id: Uuid,
        ctx: Arc<Mutex<dyn SessionContextTrait + Send + Sync>>,
    ) -> anyhow::Result<()>;
    async fn pick(
        &self,
        id: Uuid,
    ) -> anyhow::Result<Arc<Mutex<dyn SessionContextTrait + Send + Sync>>>;
}

#[async_trait]
pub trait SessionContextTrait {
    async fn closed(&self) -> bool;
    async fn close(&mut self) -> anyhow::Result<()>;
    async fn get_session_id(&self) -> Uuid;
    async fn get_session_type(&self) -> SessionTypeEnum;
}
