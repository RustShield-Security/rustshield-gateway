use clap::Parser;
use mavlink_rust_shield_gateway::{cli::Cli, config::AppConfig, logging, transport::UdpGateway};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let config = match &cli.config {
        Some(path) => AppConfig::load_from_path(path)?,
        None => AppConfig::default(),
    };
    config.validate()?;

    let log_level = cli
        .log_level
        .as_deref()
        .unwrap_or(config.logging.level.as_str());
    logging::init(log_level)?;

    tracing::info!(
        event = "gateway.start",
        transport = ?config.transport.mode,
        audit_only = config.security.audit_only,
        signing_policy = ?config.signing.policy,
        log_level,
        "gateway starting"
    );

    let gateway = UdpGateway::bind(config).await?;
    gateway.run_until_shutdown(tokio::signal::ctrl_c()).await?;

    tracing::info!(event = "gateway.shutdown", "gateway stopped");

    Ok(())
}
