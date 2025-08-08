use clap::Parser;
use genesis_web::config::{init_shared_app_state, AppConfig};
use genesis_web::{adapter, cmd::*, config};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter, Layer, Registry};

use opentelemetry::global;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;

static mut LOG_GUARD: Option<tracing_appender::non_blocking::WorkerGuard> = None;

#[tokio::main]
async fn main() {
    // step1. parse cli
    let cli = GenesisCli::parse();
    match cli.command {
        Commands::Run { config } => {
            // set config
            let config = config::parse_config(&config).await.unwrap();
            // tracing initial
            tracing_initial(&config);
            // init state
            let state = init_shared_app_state(&config).await.unwrap();
            // step2. start web
            adapter::http::server::start_http_server(&config, state)
                .await
                .unwrap();
        }
    }
}

fn tracing_initial(config: &AppConfig) {
    // ===== 先准备日志过滤规则（字符串）=====
    let mut filter_spec = std::env::var("RUST_LOG").unwrap_or_default();
    if let Some(tracing_cfg) = &config.tracing {
        if !filter_spec.is_empty() {
            filter_spec.push(',');
        }
        filter_spec.push_str(&tracing_cfg.filter);
    }
    // ===== Console 输出 =====
    let console_layer = fmt::layer()
        .pretty()
        .with_span_events(fmt::format::FmtSpan::CLOSE)
        .with_filter(EnvFilter::new(filter_spec.clone()));

    // ===== 文件输出 =====
    let file_appender = tracing_appender::rolling::daily("./", "genesis.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    unsafe {
        LOG_GUARD = Some(guard);
    }
    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .pretty()
        .with_filter(EnvFilter::new(filter_spec.clone()));

    // ===== Metrics（OTel 集成）=====
    // let meter_provider = init_metrics();
    // let metrics_layer = tracing_opentelemetry::MetricsLayer::new(meter_provider)
    //     .with_filter(EnvFilter::new(filter_spec));
    //
    let traces_provider = init_traces();
    // 加上过后会多上很多系统的日志
    // global::set_tracer_provider(traces_provider.clone());
    let tracer = traces_provider.tracer("genesis-tracer-global");
    let trace_layer = tracing_opentelemetry::layer()
        .with_tracer(tracer)
        .with_filter(EnvFilter::new(filter_spec.clone()));
    // ===== 注册所有 Layer =====
    Registry::default()
        .with(console_layer)
        .with(file_layer)
        .with(trace_layer)
        .init();
}

#[allow(dead_code)]
fn init_metrics() -> SdkMeterProvider {
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .build()
        .expect("Failed to create metric exporter");

    let meter_provider = SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .with_resource(
            Resource::builder()
                .with_service_name("genesis-web-metrics")
                .build(),
        )
        .build();
    global::set_meter_provider(meter_provider.clone());
    meter_provider
}

fn init_traces() -> SdkTracerProvider {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4317")
        .build()
        .expect("Failed to create span exporter");
    SdkTracerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_service_name("genesis-web-traces")
                .build(),
        )
        .with_batch_exporter(exporter)
        .build()
}

#[allow(dead_code)]
fn init_logs() -> opentelemetry_sdk::logs::SdkLoggerProvider {
    let exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .build()
        .expect("Failed to create log exporter");

    opentelemetry_sdk::logs::SdkLoggerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_service_name("genesis-web-logs")
                .build(),
        )
        .with_batch_exporter(exporter)
        .build()
}
