#![allow(dead_code)]
mod proto;
use crate::proto::stat::system_stat_service_server::{SystemStatService, SystemStatServiceServer};
use crate::proto::types::{Empty, SystemInfo};
use tonic::transport::Server;
use tonic::{Request, Response, Status};

#[derive(Default)]
struct MyServer {}

#[tonic::async_trait]
impl SystemStatService for MyServer {
    async fn system_info(&self, request: Request<SystemInfo>) -> Result<Response<Empty>, Status> {
        println!("got a request: {:?}", request);
        Ok(Response::new(Empty {}))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let greeter = MyServer::default();

    println!("system stat listening on {addr}");
    Server::builder()
        .add_service(SystemStatServiceServer::new(greeter))
        .serve(addr)
        .await?;
    Ok(())
}
