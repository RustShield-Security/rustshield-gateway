use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "mavlink-shield-gateway",
    version,
    about = "Security-sensitive MAVLink gateway MVP skeleton"
)]
pub struct Cli {
    /// Path to the TOML configuration file.
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Override the configured log level or tracing EnvFilter expression.
    #[arg(long, env = "MAVLINK_SHIELD_LOG")]
    pub log_level: Option<String>,
}

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, Parser};

    use super::*;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn parses_config_path_and_log_override() {
        let cli = Cli::try_parse_from([
            "mavlink-shield-gateway",
            "--config",
            "config/sitl.toml",
            "--log-level",
            "debug",
        ])
        .expect("valid CLI args");

        assert_eq!(cli.config, Some(PathBuf::from("config/sitl.toml")));
        assert_eq!(cli.log_level.as_deref(), Some("debug"));
    }
}
