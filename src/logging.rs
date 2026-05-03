use thiserror::Error;
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Debug, Error)]
pub enum LoggingError {
    #[error("invalid log filter: {0}")]
    InvalidFilter(#[from] tracing_subscriber::filter::ParseError),

    #[error("could not initialize tracing subscriber: {0}")]
    Init(String),
}

pub fn init(filter: &str) -> Result<(), LoggingError> {
    let env_filter = EnvFilter::try_new(filter)?;
    fmt()
        .with_env_filter(env_filter)
        .try_init()
        .map_err(|error| LoggingError::Init(error.to_string()))?;
    Ok(())
}
