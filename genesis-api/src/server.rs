#![allow(dead_code)]
mod proto;

use crate::proto::metrics::NodeSystemMetrics;
use crate::proto::metrics::metrics_service_server::{MetricsService, MetricsServiceServer};
use crate::proto::types::Empty;
use tonic::service::Interceptor;
use tonic::transport::{Identity, Server, ServerTlsConfig};
use tonic::{Request, Response, Status};

#[derive(Default)]
struct MyServer {}

#[tonic::async_trait]
impl MetricsService for MyServer {
    async fn system_metrics(
        &self,
        request: Request<NodeSystemMetrics>,
    ) -> Result<Response<Empty>, Status> {
        let x = request.get_ref();
        println!("Got system metrics: {:?}", x);
        Ok(Response::new(Empty {}))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let greeter = MyServer::default();
    let identity = Identity::from_pem("cert.pem", "key.pem");
    println!("system stat listening on {addr}");
    Server::builder()
        .tls_config(ServerTlsConfig::new().identity(identity))?
        //.add_service(MetricsServiceServer::new(greeter))
        .add_service(MetricsServiceServer::with_interceptor(
            greeter,
            AuthInterceptor,
        ))
        .serve(addr)
        .await?;
    Ok(())
}

#[derive(Clone)]
struct AuthInterceptor;
impl AuthInterceptor {
    fn verify(&self, _: &str) -> Result<(), String> {
        Ok(())
    }
}
impl Interceptor for AuthInterceptor {
    fn call(&mut self, req: Request<()>) -> Result<Request<()>, Status> {
        let token = req
            .metadata()
            .get("authorization")
            .and_then(|v| v.to_str().ok());
        match token {
            Some(bearer) => {
                let token = bearer
                    .strip_prefix("Bearer ")
                    .ok_or_else(|| Status::unauthenticated("invalid token format"))?;
                self.verify(token)
                    .map_err(|e| Status::unauthenticated(e.to_string()))?
            }
            None => return Err(Status::unauthenticated("missing token")),
        };
        // req.extensions_mut().insert(user);
        Ok(req)
    }
}
