use crate::proto::metrics::NodeSystemMetrics;
use crate::proto::metrics::metrics_service_client::MetricsServiceClient;
use crate::proto::types::{CpuMetrics, DiskMetrics, DiskMetricsItem, MemMetrics, SystemMetrics};

mod proto;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MetricsServiceClient::connect("http://[::1]:50051").await?;
    let request = tonic::Request::new(NodeSystemMetrics {
        uniq_id: "123456".to_string(),
        metrics: Some(SystemMetrics {
            mem: Some(MemMetrics {
                total: 111,
                available: 11,
                used: 11,
                usage: 10.0,
            }),
            cpu: Some(CpuMetrics { usage: 20.0 }),
            disk: Some(DiskMetrics {
                data: vec![DiskMetricsItem {
                    device: "111".to_string(),
                    total: 110,
                    used: 10,
                }],
            }),
        }),
    });
    let response = client.system_metrics(request).await?;
    println!("RESPONSE={response:?}");
    Ok(())
}
