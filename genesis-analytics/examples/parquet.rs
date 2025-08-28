use anyhow::Result;
use arrow::array::AsArray as _;
use datafusion::arrow::array::AsArray;
use datafusion::execution::context::SessionContext;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use polars::prelude::*;
use polars::sql::SQLContext;
use std::fs::File;

const PQ_FILE: &str = "assets/user.parquet";

#[tokio::main]
async fn main() -> Result<()> {
    read_with_parquet(PQ_FILE)?;
    read_with_data_fusion(PQ_FILE).await?;
    read_with_polars(PQ_FILE).await?;
    Ok(())
}

fn read_with_parquet(file: &str) -> Result<()> {
    let file = File::open(file)?;
    // batch_size表示一批次最大读取到行数
    // limit决定总行数,到这个数就不会再读取
    let reader = ParquetRecordBatchReaderBuilder::try_new(file)?
        .with_batch_size(8192)
        .with_limit(3)
        .build()?;

    for record_batch in reader {
        let record_batch = record_batch?;
        // 获取到原始数据字段类型为utf8,可直接转为string
        let names = record_batch.column(1).as_string::<i32>();
        for name in names {
            let name = name.unwrap();
            println!("{name:?}");
        }
    }

    Ok(())
}

async fn read_with_data_fusion(file: &str) -> Result<()> {
    let ctx = SessionContext::new();
    ctx.register_parquet("xxx", file, Default::default())
        .await?;

    let ret = ctx
        .sql("SELECT email::text name, username::text username FROM xxx limit 3")
        .await?
        .collect()
        .await?;
    for batch in ret {
        // 获取到原始数据类型为utf8View,需转换为string_view
        let names = batch.column(0).as_string_view();
        let usernames = batch.column(1).as_string_view();
        for (name, username) in names.iter().zip(usernames.iter()) {
            let (name, username) = (name.unwrap(), username.unwrap());
            println!("{name} {username}");
        }
    }
    Ok(())
}

async fn read_with_polars(file: &str) -> Result<()> {
    let df = LazyFrame::scan_parquet(PlPath::new(file), Default::default())?;
    // 需要放在单独的线程处理
    // polars处理的时候会尝试创建一个runtime,会阻塞核心线程
    // 由于外部已经启用了tokio的线程,所以会报错
    // spawn_blocking把阻塞操作移动到 专门的阻塞线程池,保证 async 核心线程不被阻塞
    let df: DataFrame = tokio::task::spawn_blocking(move || {
        // 这里是同步操作，放到 spawn_blocking 线程池
        let mut ctx = SQLContext::new();
        ctx.register("user", df);
        ctx.execute("SELECT email::text, name::text FROM user")
            .unwrap()
            .collect()
            .unwrap()
    })
    .await?;
    println!("{df:?}");
    Ok(())
}
