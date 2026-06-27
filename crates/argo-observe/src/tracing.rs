use tracing_subscriber::EnvFilter;

pub fn init_tracing(enabled: bool, backend: &str, _endpoint: &str) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    if enabled && backend == "otlp" {
        // OTel setup would go here in production
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .json()
            .init();
    } else if enabled && backend == "stdout" {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .json()
            .init();
    }
}
