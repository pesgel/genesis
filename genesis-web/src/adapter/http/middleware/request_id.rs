use crate::error::AppError;
use axum::body::Body;
use axum::http::{HeaderValue, Request};
use axum::middleware::Next;
use axum::response::Response;
use tracing::log::warn;
use uuid::Uuid;

const X_REQUEST_ID: &str = "x-request-id";
pub async fn request_id(mut req: Request<Body>, next: Next) -> Result<Response, AppError> {
    let id = match req.headers().get(X_REQUEST_ID) {
        None => {
            let request_id = Uuid::new_v4().to_string();
            match request_id.parse() {
                Ok(uuid) => {
                    req.headers_mut().insert(X_REQUEST_ID, uuid);
                }
                Err(e) => {
                    warn!("parse uuid error :{e}");
                }
            }
            request_id.as_bytes().to_vec()
        }
        Some(v) => v.as_bytes().to_vec(),
    };
    let mut res = next.run(req).await;
    match HeaderValue::from_bytes(&id) {
        Ok(v) => {
            res.headers_mut().insert(X_REQUEST_ID, v);
        }
        Err(e) => warn!("parse header error :{e}"),
    }
    Ok(res)
}
