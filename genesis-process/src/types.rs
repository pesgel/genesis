//! types

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type AsyncMatchFn = Arc<
    dyn Fn(
            Arc<RwLock<HashMap<String, String>>>,
        )
            -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<bool>> + Send>>
        + Send
        + Sync,
>;
