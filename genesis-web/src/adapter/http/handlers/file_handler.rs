use crate::adapter::ResponseSuccess;
use crate::error::AppError;
use axum::extract::Multipart;
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;

pub async fn file_stream_upload(mut multipart: Multipart) -> Result<ResponseSuccess, AppError> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        if field.name() == Some("file") {
            let file_name = field.file_name().unwrap_or("upload.bin").to_string();
            let mut file = tokio::fs::File::create(file_name).await.unwrap();
            let mut field = field; // 它本身就是一个 Stream
            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                file.write_all(&data).await.unwrap();
            }
        }
    }
    Ok(ResponseSuccess::default())
}
