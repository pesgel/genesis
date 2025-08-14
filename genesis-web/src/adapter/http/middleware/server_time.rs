//! create layer and middleware

use axum::extract::Request;
use axum::http::HeaderValue;
use axum::response::Response;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tracing::warn;

#[derive(Clone, Debug)]
pub struct ServerTimeLayer;

impl<S> Layer<S> for ServerTimeLayer {
    type Service = ServerTimeMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ServerTimeMiddleware { inner }
    }
}

#[derive(Clone, Debug)]
pub struct ServerTimeMiddleware<S> {
    inner: S,
}

/// Request: axum_core::extract::Request
impl<S> Service<Request> for ServerTimeMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let future = self.inner.call(req);
        let start = tokio::time::Instant::now();
        Box::pin(async move {
            let mut response = future.await?;
            let elapsed = format!("{}us", start.elapsed().as_micros());
            match HeaderValue::from_str(&elapsed) {
                Ok(value) => {
                    response.headers_mut().insert("x-server-time", value);
                }
                Err(e) => warn!("parse server time error: {:?}", e),
            }
            Ok(response)
        })
    }
}
