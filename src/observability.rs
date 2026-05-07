use std::{io, net::SocketAddr, sync::Arc, time::Instant};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
    task::JoinHandle,
};

use crate::metrics::GatewayCounters;

const MAX_REQUEST_BYTES: usize = 1024;

pub struct ReadonlyEndpoint {
    pub local_addr: SocketAddr,
    pub task: JoinHandle<()>,
}

pub async fn spawn_readonly_endpoint(
    bind_addr: SocketAddr,
    counters: Arc<Mutex<GatewayCounters>>,
    started_at: Instant,
) -> io::Result<ReadonlyEndpoint> {
    let listener = TcpListener::bind(bind_addr).await?;
    let local_addr = listener.local_addr()?;
    let task = tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, _peer)) => {
                    let counters = Arc::clone(&counters);
                    tokio::spawn(async move {
                        if let Err(error) = handle_connection(stream, counters, started_at).await {
                            tracing::warn!(
                                event = "observability.readonly_error",
                                reason = %error,
                                "read-only observability request failed"
                            );
                        }
                    });
                }
                Err(error) => {
                    tracing::warn!(
                        event = "observability.readonly_error",
                        reason = %error,
                        "read-only observability listener failed"
                    );
                    break;
                }
            }
        }
    });

    Ok(ReadonlyEndpoint { local_addr, task })
}

async fn handle_connection(
    mut stream: TcpStream,
    counters: Arc<Mutex<GatewayCounters>>,
    started_at: Instant,
) -> io::Result<()> {
    let mut request = [0_u8; MAX_REQUEST_BYTES];
    let len = stream.read(&mut request).await?;
    let path = request_path(&request[..len]).unwrap_or("/");
    let counters = *counters.lock().await;
    let uptime_seconds = started_at.elapsed().as_secs();

    let response = match path {
        "/healthz" => http_response(
            "200 OK",
            "application/json; charset=utf-8",
            &render_health_json(&counters, uptime_seconds),
        ),
        "/metrics" => http_response(
            "200 OK",
            "text/plain; version=0.0.4; charset=utf-8",
            &render_prometheus_metrics(&counters, uptime_seconds),
        ),
        _ => http_response(
            "404 Not Found",
            "application/json; charset=utf-8",
            r#"{"status":"not_found"}"#,
        ),
    };

    stream.write_all(response.as_bytes()).await?;
    stream.shutdown().await
}

fn request_path(request: &[u8]) -> Option<&str> {
    let first_line = std::str::from_utf8(request).ok()?.lines().next()?;
    let mut parts = first_line.split_ascii_whitespace();
    let method = parts.next()?;
    let path = parts.next()?;
    if method == "GET" {
        Some(path)
    } else {
        None
    }
}

fn http_response(status: &str, content_type: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\ncontent-type: {content_type}\r\ncontent-length: {}\r\ncache-control: no-store\r\nconnection: close\r\n\r\n{body}",
        body.len()
    )
}

pub fn render_health_json(counters: &GatewayCounters, uptime_seconds: u64) -> String {
    format!(
        "{{\"status\":\"ok\",\"read_only\":true,\"uptime_seconds\":{uptime_seconds},\"packets_received_total\":{},\"packets_forwarded_total\":{},\"packets_blocked_total\":{},\"packets_parse_error_total\":{}}}",
        counters.packets_received_total,
        counters.packets_forwarded_total,
        counters.packets_blocked_total,
        counters.packets_parse_error_total,
    )
}

pub fn render_prometheus_metrics(counters: &GatewayCounters, uptime_seconds: u64) -> String {
    let mut output = String::new();
    push_metric(&mut output, "gateway_uptime_seconds", uptime_seconds);
    push_metric(
        &mut output,
        "packets_received_total",
        counters.packets_received_total,
    );
    push_metric(
        &mut output,
        "packets_forwarded_total",
        counters.packets_forwarded_total,
    );
    push_metric(
        &mut output,
        "packets_blocked_total",
        counters.packets_blocked_total,
    );
    push_metric(
        &mut output,
        "packets_parse_error_total",
        counters.packets_parse_error_total,
    );
    push_metric(
        &mut output,
        "packets_signed_observed_total",
        counters.packets_signed_observed_total,
    );
    push_metric(
        &mut output,
        "packets_signed_valid_total",
        counters.packets_signed_valid_total,
    );
    push_metric(
        &mut output,
        "packets_signed_invalid_total",
        counters.packets_signed_invalid_total,
    );
    push_metric(
        &mut output,
        "packets_unsigned_rejected_total",
        counters.packets_unsigned_rejected_total,
    );
    push_metric(
        &mut output,
        "signing_replay_rejected_total",
        counters.signing_replay_rejected_total,
    );
    push_metric(
        &mut output,
        "setup_signing_observed_total",
        counters.setup_signing_observed_total,
    );
    push_metric(
        &mut output,
        "shadow_policy_would_block_total",
        counters.shadow_policy_would_block_total,
    );
    push_metric(
        &mut output,
        "shadow_signing_would_reject_total",
        counters.shadow_signing_would_reject_total,
    );
    push_metric(
        &mut output,
        "shadow_unsigned_critical_total",
        counters.shadow_unsigned_critical_total,
    );
    push_metric(
        &mut output,
        "shadow_invalid_signature_total",
        counters.shadow_invalid_signature_total,
    );
    push_metric(
        &mut output,
        "shadow_replay_total",
        counters.shadow_replay_total,
    );
    push_metric(
        &mut output,
        "commands_critical_observed_total",
        counters.commands_critical_observed_total,
    );
    push_metric(
        &mut output,
        "processing_latency_samples",
        counters.processing_latency_samples,
    );
    push_metric(
        &mut output,
        "processing_latency_total_us",
        counters.processing_latency_total_us,
    );
    push_metric(
        &mut output,
        "processing_latency_max_us",
        counters.processing_latency_max_us,
    );
    push_metric(
        &mut output,
        "parse_latency_samples",
        counters.parse_latency_samples,
    );
    push_metric(
        &mut output,
        "parse_latency_total_us",
        counters.parse_latency_total_us,
    );
    push_metric(
        &mut output,
        "parse_latency_max_us",
        counters.parse_latency_max_us,
    );
    push_metric(
        &mut output,
        "policy_latency_samples",
        counters.policy_latency_samples,
    );
    push_metric(
        &mut output,
        "policy_latency_total_us",
        counters.policy_latency_total_us,
    );
    push_metric(
        &mut output,
        "policy_latency_max_us",
        counters.policy_latency_max_us,
    );
    output
}

fn push_metric(output: &mut String, name: &str, value: u64) {
    output.push_str(name);
    output.push(' ');
    output.push_str(&value.to_string());
    output.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpStream;

    #[test]
    fn renders_health_json_without_sensitive_fields() {
        let counters = GatewayCounters {
            packets_received_total: 7,
            packets_forwarded_total: 5,
            packets_blocked_total: 1,
            packets_parse_error_total: 1,
            ..GatewayCounters::default()
        };

        let output = render_health_json(&counters, 42);

        assert!(output.contains(r#""status":"ok""#));
        assert!(output.contains(r#""read_only":true"#));
        assert!(output.contains(r#""packets_blocked_total":1"#));
        assert!(!output.contains("payload"));
        assert!(!output.contains("secret"));
        assert!(!output.contains("key"));
    }

    #[test]
    fn renders_prometheus_metrics_without_payload_or_key_material() {
        let counters = GatewayCounters {
            packets_received_total: 3,
            packets_signed_valid_total: 2,
            signing_replay_rejected_total: 1,
            setup_signing_observed_total: 1,
            shadow_policy_would_block_total: 1,
            shadow_signing_would_reject_total: 2,
            processing_latency_max_us: 99,
            ..GatewayCounters::default()
        };

        let output = render_prometheus_metrics(&counters, 11);

        assert!(output.contains("gateway_uptime_seconds 11\n"));
        assert!(output.contains("packets_received_total 3\n"));
        assert!(output.contains("packets_signed_valid_total 2\n"));
        assert!(output.contains("signing_replay_rejected_total 1\n"));
        assert!(output.contains("setup_signing_observed_total 1\n"));
        assert!(output.contains("shadow_policy_would_block_total 1\n"));
        assert!(output.contains("shadow_signing_would_reject_total 2\n"));
        assert!(!output.contains("payload"));
        assert!(!output.contains("signature="));
        assert!(!output.contains("signature_value"));
        assert!(!output.contains("signature_bytes"));
        assert!(!output.contains("key"));
        assert!(!output.contains("SETUP_SIGNING_DATA"));
    }

    #[test]
    fn parses_only_get_request_path() {
        assert_eq!(
            request_path(b"GET /metrics HTTP/1.1\r\n\r\n"),
            Some("/metrics")
        );
        assert_eq!(request_path(b"POST /metrics HTTP/1.1\r\n\r\n"), None);
    }

    #[tokio::test]
    async fn exposes_metrics_over_readonly_http() {
        let counters = Arc::new(Mutex::new(GatewayCounters {
            packets_received_total: 9,
            packets_blocked_total: 2,
            ..GatewayCounters::default()
        }));
        let endpoint = spawn_readonly_endpoint(
            "127.0.0.1:0".parse().expect("valid bind"),
            Arc::clone(&counters),
            Instant::now(),
        )
        .await
        .expect("endpoint binds");

        let mut stream = TcpStream::connect(endpoint.local_addr)
            .await
            .expect("endpoint accepts local connection");
        stream
            .write_all(b"GET /metrics HTTP/1.1\r\nhost: localhost\r\n\r\n")
            .await
            .expect("request writes");

        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .await
            .expect("response reads");
        endpoint.task.abort();

        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(!response.contains("access-control-allow-origin"));
        assert!(response.contains("packets_received_total 9\n"));
        assert!(response.contains("packets_blocked_total 2\n"));
        assert!(!response.contains("payload"));
        assert!(!response.contains("key"));
    }

    #[tokio::test]
    async fn returns_not_found_for_unknown_readonly_path() {
        let counters = Arc::new(Mutex::new(GatewayCounters::default()));
        let endpoint = spawn_readonly_endpoint(
            "127.0.0.1:0".parse().expect("valid bind"),
            Arc::clone(&counters),
            Instant::now(),
        )
        .await
        .expect("endpoint binds");

        let mut stream = TcpStream::connect(endpoint.local_addr)
            .await
            .expect("endpoint accepts local connection");
        stream
            .write_all(b"GET /unknown HTTP/1.1\r\nhost: localhost\r\n\r\n")
            .await
            .expect("request writes");

        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .await
            .expect("response reads");
        endpoint.task.abort();

        assert!(response.starts_with("HTTP/1.1 404 Not Found"));
        assert!(response.contains(r#"{"status":"not_found"}"#));
    }
}
