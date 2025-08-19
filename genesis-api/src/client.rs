use crate::proto::stat::system_stat_service_client::SystemStatServiceClient;
use crate::proto::types::{CpuStat, SystemInfo};

mod proto;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = SystemStatServiceClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(SystemInfo {
        mem: None,
        cpu: Some(CpuStat {
            usage: 32f32,
            uniq_id: "123".to_string(),
        }),
        disk: None,
    });
    let response = client.system_info(request).await?;
    println!("RESPONSE={response:?}");
    Ok(())
}
