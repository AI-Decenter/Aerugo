use tracing::Subscriber;
use tracing_subscriber::{
    fmt::{format::FmtSpan, time},
    EnvFilter,
};

pub fn setup_logging() -> impl Subscriber {
    tracing_subscriber::fmt()
        .with_timer(time::time())
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE)
        .json()
        .finish() // Changed from .into_make_subscriber()
}