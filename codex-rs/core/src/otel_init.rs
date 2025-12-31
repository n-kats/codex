use crate::config::Config;
use crate::config::types::OtelExporterKind as Kind;
use crate::config::types::OtelHttpProtocol as Protocol;
use crate::default_client::originator;
use codex_otel::config::OtelExporter;
use codex_otel::config::OtelHttpProtocol;
use codex_otel::config::OtelSettings;
use codex_otel::config::OtelTlsConfig as OtelTlsSettings;
use codex_otel::otel_provider::OtelProvider;
use base64::Engine;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;

fn append_otel_headers_to_debug_log(config: &Config) {
    let enabled = !matches!(config.otel.exporter, Kind::None)
        || !matches!(config.otel.trace_exporter, Kind::None);
    if !enabled {
        return;
    }

    let Ok(log_dir) = crate::config::log_dir(config) else {
        return;
    };
    let _ = std::fs::create_dir_all(&log_dir);
    let path = log_dir.join("debug.log");

    fn describe_headers(headers: &std::collections::HashMap<String, String>) -> String {
        let mut keys: Vec<_> = headers.keys().collect();
        keys.sort();

        let mut out = String::new();
        for key in keys {
            let Some(value) = headers.get(key) else {
                continue;
            };

            if key.eq_ignore_ascii_case("authorization") {
                let escaped = value.escape_default().to_string();
                let (scheme, credentials) = value.split_once(' ').unwrap_or((value, ""));
                let has_whitespace_in_credentials =
                    credentials.chars().any(|c| c.is_whitespace());

                let mut invalid = String::new();
                if scheme == "Basic" {
                    for ch in credentials.chars() {
                        let ok = ch.is_ascii_alphanumeric() || ch == '+' || ch == '/' || ch == '=';
                        if !ok && !invalid.contains(ch) {
                            invalid.push(ch);
                        }
                    }
                }

                let preview = if escaped.len() <= 64 {
                    escaped
                } else {
                    format!("{}...{}", &escaped[..32], &escaped[escaped.len() - 32..])
                };

                let mut decode_summary = String::new();
                if scheme == "Basic" && !credentials.is_empty() {
                    match base64::prelude::BASE64_STANDARD.decode(credentials) {
                        Ok(decoded) => {
                            let decoded_str = String::from_utf8_lossy(&decoded);
                            let (user, pass) = decoded_str.split_once(':').unwrap_or(("", ""));
                            decode_summary = format!(
                                " decoded_ok=true user_len={} pass_len={} user_pk_lf={} pass_sk_lf={}",
                                user.len(),
                                pass.len(),
                                user.starts_with("pk-lf-"),
                                pass.starts_with("sk-lf-")
                            );
                        }
                        Err(_) => {
                            decode_summary = " decoded_ok=false".to_string();
                        }
                    }
                }

                out.push_str(&format!(
                    "  {key}=(len={} scheme={scheme} whitespace_in_credentials={} invalid_base64_chars={:?} preview={preview}{decode_summary})\n",
                    value.len(),
                    has_whitespace_in_credentials,
                    invalid
                ));
                out.push_str(&format!("  {key}.escaped={}\n", value.escape_default()));
            } else {
                out.push_str(&format!("  {key}={}\n", value.escape_default()));
            }
        }
        out
    }

    let mut lines = Vec::new();
    lines.push(format!(
        "[otel] exporter={:?} trace_exporter={:?}\n",
        config.otel.exporter, config.otel.trace_exporter
    ));

    match &config.otel.exporter {
        Kind::OtlpHttp { endpoint, headers, .. } | Kind::OtlpGrpc { endpoint, headers, .. } => {
            lines.push(format!("[otel] exporter endpoint={endpoint}\n"));
            lines.push(describe_headers(headers));
        }
        Kind::None => {}
    }
    match &config.otel.trace_exporter {
        Kind::OtlpHttp { endpoint, headers, .. } | Kind::OtlpGrpc { endpoint, headers, .. } => {
            lines.push(format!("[otel] trace_exporter endpoint={endpoint}\n"));
            lines.push(describe_headers(headers));
        }
        Kind::None => {}
    }

    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    let _ = file.write_all(lines.concat().as_bytes());
}

/// Build an OpenTelemetry provider from the app Config.
///
/// Returns `None` when OTEL export is disabled.
pub fn build_provider(
    config: &Config,
    service_version: &str,
) -> Result<Option<OtelProvider>, Box<dyn Error>> {
    append_otel_headers_to_debug_log(config);
    let to_otel_exporter = |kind: &Kind| match kind {
        Kind::None => OtelExporter::None,
        Kind::OtlpHttp {
            endpoint,
            headers,
            protocol,
            tls,
        } => {
            let protocol = match protocol {
                Protocol::Json => OtelHttpProtocol::Json,
                Protocol::Binary => OtelHttpProtocol::Binary,
            };

            OtelExporter::OtlpHttp {
                endpoint: endpoint.clone(),
                headers: headers
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
                protocol,
                tls: tls.as_ref().map(|config| OtelTlsSettings {
                    ca_certificate: config.ca_certificate.clone(),
                    client_certificate: config.client_certificate.clone(),
                    client_private_key: config.client_private_key.clone(),
                }),
            }
        }
        Kind::OtlpGrpc {
            endpoint,
            headers,
            tls,
        } => OtelExporter::OtlpGrpc {
            endpoint: endpoint.clone(),
            headers: headers
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            tls: tls.as_ref().map(|config| OtelTlsSettings {
                ca_certificate: config.ca_certificate.clone(),
                client_certificate: config.client_certificate.clone(),
                client_private_key: config.client_private_key.clone(),
            }),
        },
    };

    let exporter = to_otel_exporter(&config.otel.exporter);
    let trace_exporter = to_otel_exporter(&config.otel.trace_exporter);

    OtelProvider::from(&OtelSettings {
        service_name: originator().value.to_owned(),
        service_version: service_version.to_string(),
        codex_home: config.codex_home.clone(),
        environment: config.otel.environment.to_string(),
        exporter,
        trace_exporter,
    })
}

/// Filter predicate for exporting only Codex-owned events via OTEL.
/// Keeps events that originated from codex_otel module
pub fn codex_export_filter(meta: &tracing::Metadata<'_>) -> bool {
    meta.target().starts_with("codex_otel")
}
