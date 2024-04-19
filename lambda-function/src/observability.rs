use lambda_runtime::Error;
use opentelemetry::trace::{TraceId,TraceContextExt};
use opentelemetry::{Context,KeyValue};
use opentelemetry_sdk::{
    trace::{self},
    Resource,
};

use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, fmt::format, EnvFilter};
use opentelemetry_otlp::WithExportConfig;

pub fn init_observability() -> Result<(), Error> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_timeout(std::time::Duration::from_secs(3)),
        )
        .with_trace_config(
            trace::config()
                .with_resource(Resource::new(vec![KeyValue::new("service.name", "lambda")])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;
    let fmt_layer = fmt::layer().event_format(format().json());
    let filter_layer = EnvFilter::from_default_env();
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(telemetry_layer)
        .init();
    Ok(())
}


pub fn get_trace_id() -> TraceId {
    Context::current().span().span_context().trace_id()
}

use opentelemetry::trace::SpanContext;
use opentelemetry_aws::trace::span_context_from_str;
pub fn get_span_context_from_environment_var() -> Result<SpanContext, &'static str> {
    let xray_trace_id =
        std::env::var("_X_AMZN_TRACE_ID").map_err(|_| "_X_AMZN_TRACE_ID not set")?;
    span_context_from_str(&xray_trace_id).ok_or("Invalid _X_AMZN_TRACE_ID")
}
