//! parquet
use clap::{Args, Parser, Subcommand, arg, command};
use datafusion::arrow::util::pretty::pretty_format_batches;
use datafusion::dataframe::DataFrame;
use datafusion::prelude::{ParquetReadOptions, SessionContext};
use enum_dispatch::enum_dispatch;
use reedline_repl_rs::clap::ArgMatches;
use reedline_repl_rs::{AsyncCallBackMap, Repl, Result};
use std::ops::Deref;
use std::process;
use std::sync::Arc;
use tokio::select;
use tokio::sync::oneshot::Receiver;
use tokio::sync::{mpsc, oneshot};

#[tokio::main]
async fn main() -> Result<()> {
    let (sx, mut rx) = mpsc::unbounded_channel();
    let context = GlobalContext { inner: sx };
    let mut callbacks = AsyncCallBackMap::<GlobalContext, reedline_repl_rs::Error>::new();
    callbacks.insert("sql".to_string(), |args, context| {
        Box::pin(sql(args, context))
    });
    callbacks.insert("base".to_string(), |args, context| {
        Box::pin(base(args, context))
    });
    callbacks.insert("connect".to_string(), |args, context| {
        Box::pin(connect(args, context))
    });

    callbacks.insert("schema".to_string(), |args, context| {
        Box::pin(schema(args, context))
    });

    let backend = DataFusionBackend::new();
    let mut repl = Repl::new(context)
        .with_banner("Welcome to MyApp")
        .with_async_derived::<MyApp>(callbacks);
    tokio::spawn(async move {
        let mut bk = backend.clone();
        loop {
            select! {
                value = rx.recv() => {
                    match value {
                        Some(msg) => {
                            let res = msg.cmd.execute_now(&mut bk).await;
                            match res {
                                Ok(data) => {
                                    msg.sender.send(Some(data)).unwrap();
                                },
                                Err(e) => {
                                    println!("error executing command:{e}");
                                }
                            }
                        },
                        None => {
                            println!("receive none, break");
                            break;
                        },
                    }
                }
            }
        }
    });
    repl.run_async().await
}

#[derive(Parser, Debug)]
#[command(name = "App", version = "v0.1.0", about = "connect app")]
pub struct MyApp {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Base {
    #[command(about = "exit")]
    Exit,
}

#[enum_dispatch]
trait CmdExecutor {
    async fn execute_now<T: Backend>(self, backend: &mut T) -> anyhow::Result<String>;
}

#[derive(Debug, Subcommand)]
#[enum_dispatch(CmdExecutor)]
pub enum Commands {
    #[command(subcommand)]
    Base(Base),
    #[command(about = "connect opts")]
    Connect(ConnectOpts),
    Sql(SqlOpts),
    Schema(SchemaOpts),
}

impl CmdExecutor for SqlOpts {
    async fn execute_now<T: Backend>(self, backend: &mut T) -> anyhow::Result<String> {
        let be = backend.sql(&self.sql).await?;
        be.display().await
    }
}

impl CmdExecutor for ConnectOpts {
    async fn execute_now<T: Backend>(self, backend: &mut T) -> anyhow::Result<String> {
        backend
            .connect(&ConnectOpts {
                conn: self.conn,
                table: self.table,
            })
            .await?;
        Ok("connected successfully".to_string())
    }
}

impl CmdExecutor for Base {
    async fn execute_now<T: Backend>(self, _: &mut T) -> anyhow::Result<String> {
        match self {
            Base::Exit => process::exit(1),
        }
    }
}

impl CmdExecutor for SchemaOpts {
    async fn execute_now<T: Backend>(self, backend: &mut T) -> anyhow::Result<String> {
        let x = backend.schema(&self.schema).await?;
        x.display().await
    }
}
#[derive(Debug, Args)]
pub struct SqlOpts {
    #[arg(help = "connect str")]
    sql: String,
}
#[derive(Debug, Args)]
pub struct SchemaOpts {
    #[arg(help = "schema str")]
    schema: String,
}

#[derive(Debug, Args)]
pub struct ConnectOpts {
    #[arg(help = "connect str")]
    conn: String,
    #[arg(help = "connect table")]
    table: String,
}

async fn connect(args: ArgMatches, context: &mut GlobalContext) -> Result<Option<String>> {
    let conn = args
        .get_one::<String>("conn")
        .expect("no conn found")
        .to_owned();
    let table = args
        .get_one::<String>("table")
        .expect("no table found")
        .to_owned();
    let (sx, rx) = oneshot::channel();
    let msg = Message {
        cmd: Commands::Connect(ConnectOpts { conn, table }),
        sender: sx,
    };
    send_and_wait(msg, rx, context).await
}

async fn send_and_wait(
    msg: Message,
    rx: Receiver<Option<String>>,
    context: &mut GlobalContext,
) -> Result<Option<String>> {
    if let Err(e) = context.send(msg) {
        return Ok(Some(format!("send msg error:{e}")));
    };
    match rx.await {
        Ok(data) => Ok(data),
        Err(e) => Ok(Some(format!("rx receive msg error:{e}"))),
    }
}

async fn sql(args: ArgMatches, context: &mut GlobalContext) -> Result<Option<String>> {
    let sql = args
        .get_one::<String>("sql")
        .expect("no sql found")
        .to_owned();
    let (sx, rx) = oneshot::channel();
    let msg = Message {
        cmd: Commands::Sql(SqlOpts { sql }),
        sender: sx,
    };
    send_and_wait(msg, rx, context).await
}

async fn schema(args: ArgMatches, context: &mut GlobalContext) -> Result<Option<String>> {
    let schema = args
        .get_one::<String>("schema")
        .expect("no schema found")
        .to_owned();
    let (sx, rx) = oneshot::channel();
    let msg = Message {
        cmd: Commands::Schema(SchemaOpts { schema }),
        sender: sx,
    };
    send_and_wait(msg, rx, context).await
}

async fn base<T>(args: ArgMatches, _context: &mut T) -> Result<Option<String>> {
    match args.subcommand() {
        Some(("exit", _sub_matches)) => process::exit(1),
        _ => panic!("unknown base group command {:?}", args.subcommand_name()),
    }
}

trait ReplDisplay {
    // async fn display(self) -> anyhow::Result<String>;
    fn display(self) -> impl std::future::Future<Output = anyhow::Result<String>> + Send;
}
trait Backend {
    async fn connect(&mut self, opts: &ConnectOpts) -> anyhow::Result<()>;
    async fn schema(&self, name: &str) -> anyhow::Result<impl ReplDisplay>;
    async fn sql(&self, sql: &str) -> anyhow::Result<impl ReplDisplay>;
}

impl ReplDisplay for DataFrame {
    async fn display(self) -> anyhow::Result<String> {
        let batches = self.collect().await?;
        let data = pretty_format_batches(&batches)?;
        Ok(data.to_string())
    }
}

impl Backend for DataFusionBackend {
    async fn connect(&mut self, opts: &ConnectOpts) -> anyhow::Result<()> {
        let par = ParquetReadOptions::new();
        self.register_parquet(&opts.table, &opts.conn, par).await?;
        Ok(())
    }

    async fn schema(&self, name: &str) -> anyhow::Result<impl ReplDisplay> {
        let df = self.0.sql(&format!("DESCRIBE {name}")).await?;
        Ok(df)
    }

    async fn sql(&self, sql: &str) -> anyhow::Result<impl ReplDisplay> {
        let df = self.0.sql(sql).await?;
        Ok(df)
    }
}

#[derive(Clone)]
struct DataFusionBackend(Arc<SessionContext>);

impl DataFusionBackend {
    fn new() -> Self {
        let ctx = SessionContext::new();
        Self(Arc::new(ctx))
    }
}
impl Deref for DataFusionBackend {
    type Target = SessionContext;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

struct GlobalContext {
    inner: mpsc::UnboundedSender<Message>,
}

impl Deref for GlobalContext {
    type Target = mpsc::UnboundedSender<Message>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug)]
struct Message {
    // 发送command到全局处理
    cmd: Commands,
    sender: oneshot::Sender<Option<String>>,
}
