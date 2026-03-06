#![allow(non_snake_case)]

use crate::OtelManager;
use codex_protocol::protocol::SessionSource;
use codex_protocol::protocol::W3cTraceContext;
use opentelemetry::trace::TraceContextExt;
use pretty_assertions::assert_eq;
use tracing::Span;

#[test]
fn custom__otel_traceparent__apply_traceparent_parentは既存のsession_parent_contextを保持する() {
    let manager = OtelManager::new(
        codex_protocol::ThreadId::new(),
        "gpt-5",
        "gpt-5",
        None,
        None,
        None,
        "codex".to_string(),
        false,
        "xterm-256color".to_string(),
        SessionSource::Exec,
    );

    let expected_context = crate::trace_context::context_from_w3c_trace_context(&W3cTraceContext {
        traceparent: Some("00-00000000000000000000000000000011-0000000000000022-01".to_string()),
        tracestate: Some("vendor=value".to_string()),
    })
    .expect("valid trace context");
    *manager
        .session_parent_context
        .write()
        .expect("lock session parent") = Some(expected_context.clone());

    manager.apply_traceparent_parent(&Span::none());

    let stored_context = manager
        .session_parent_context
        .read()
        .expect("lock session parent")
        .clone()
        .expect("stored context");
    let stored_span = stored_context.span();
    let stored_span_context = stored_span.span_context();
    let expected_span = expected_context.span();
    let expected_span_context = expected_span.span_context();

    assert_eq!(
        stored_span_context.trace_id(),
        expected_span_context.trace_id()
    );
    assert_eq!(
        stored_span_context.span_id(),
        expected_span_context.span_id()
    );
    assert_eq!(
        stored_span_context.is_remote(),
        expected_span_context.is_remote()
    );
}

#[test]
fn custom__otel_traceparent__attach_session_parentはspan_contextでsession_parent_contextを上書きする()
 {
    let manager = OtelManager::new(
        codex_protocol::ThreadId::new(),
        "gpt-5",
        "gpt-5",
        None,
        None,
        None,
        "codex".to_string(),
        false,
        "xterm-256color".to_string(),
        SessionSource::Exec,
    );

    let existing_context = crate::trace_context::context_from_w3c_trace_context(&W3cTraceContext {
        traceparent: Some("00-00000000000000000000000000000033-0000000000000044-01".to_string()),
        tracestate: None,
    })
    .expect("valid trace context");
    *manager
        .session_parent_context
        .write()
        .expect("lock session parent") = Some(existing_context);

    manager.attach_session_parent(&Span::none());

    let stored_context = manager
        .session_parent_context
        .read()
        .expect("lock session parent")
        .clone()
        .expect("stored context");
    assert!(
        !stored_context.span().span_context().is_valid(),
        "expected Span::none() context to replace the previous stored context"
    );
}
