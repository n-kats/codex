use std::sync::Mutex;

// These integration tests exercise process-global OTEL state. Keep them out of
// one another's setup and teardown while allowing unrelated tests to run in
// parallel.
pub(crate) static GLOBAL_OTEL_STATE_LOCK: Mutex<()> = Mutex::new(());

#[path = "buffered_operations_tests.rs"]
mod buffered_operations;
mod manager_metrics;
mod otel_export_routing_policy;
mod otlp_http_loopback;
mod runtime_summary;
mod send;
mod snapshot;
mod timing;
mod validation;
