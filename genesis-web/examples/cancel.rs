use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() {
    let token = CancellationToken::new();
    let child_token = token.child_token();

    let task = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = child_token.cancelled() => {
                    println!("receive sig");
                    break;
                }
                _ = sleep(Duration::from_secs(1)) => {
                    println!("working...");
                }
            }
        }
    });

    sleep(Duration::from_secs(3)).await;
    token.cancel(); // 发送取消信号
    task.await.unwrap();
}
