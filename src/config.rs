use std::{
    fs,
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
};

use serde::Deserialize;
use thiserror::Error;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("could not read config file {path}: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid TOML config: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("invalid configuration: {0}")]
    Validation(String),
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    pub transport: TransportConfig,
    pub udp: UdpConfig,
    #[serde(default)]
    pub serial: SerialConfig,
    pub security: SecurityConfig,
    #[serde(default)]
    pub signing: GatewaySigningConfig,
    pub crypto: CryptoConfig,
    pub logging: LoggingConfig,
    pub metrics: MetricsConfig,
}

impl AppConfig {
    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path_ref = path.as_ref();
        let input = fs::read_to_string(path_ref).map_err(|source| ConfigError::Read {
            path: path_ref.display().to_string(),
            source,
        })?;
        Self::load_from_str(&input)
    }

    pub fn load_from_str(input: &str) -> Result<Self, ConfigError> {
        let config: Self = toml::from_str(input)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.udp.listen_gcs == self.udp.listen_vehicle {
            return Err(ConfigError::Validation(
                "udp.listen_gcs and udp.listen_vehicle must use different socket addresses"
                    .to_string(),
            ));
        }

        if self.transport.mode == TransportMode::Serial {
            self.serial.validate()?;
        }

        if self.udp.read_timeout_ms == 0 || self.udp.read_timeout_ms > 10_000 {
            return Err(ConfigError::Validation(
                "udp.read_timeout_ms must be between 1 and 10000 milliseconds".to_string(),
            ));
        }

        if self.udp.max_datagram_size < 8 || self.udp.max_datagram_size > 4096 {
            return Err(ConfigError::Validation(
                "udp.max_datagram_size must be between 8 and 4096 bytes".to_string(),
            ));
        }

        if self.security.certified_ips.is_empty() {
            return Err(ConfigError::Validation(
                "security.certified_ips must contain at least one exact IP".to_string(),
            ));
        }

        if self.security.unknown_mode_policy == UnknownModePolicy::Allow
            && self.security.block_arm_in_auto_mode
        {
            return Err(ConfigError::Validation(
                "unknown_mode_policy=allow is not accepted while critical command blocking is enabled"
                    .to_string(),
            ));
        }

        if self.logging.payload_logging {
            return Err(ConfigError::Validation(
                "payload_logging=true is rejected in the MVP skeleton".to_string(),
            ));
        }

        self.signing.validate()?;

        if let Some(readonly_bind) = self.metrics.readonly_bind {
            if !readonly_bind.ip().is_loopback() {
                return Err(ConfigError::Validation(
                    "metrics.readonly_bind must use a loopback address in this phase".to_string(),
                ));
            }
        }

        EnvFilter::try_new(&self.logging.level).map_err(|source| {
            ConfigError::Validation(format!(
                "logging.level is not a valid tracing filter: {source}"
            ))
        })?;

        if self.crypto.enabled {
            return Err(ConfigError::Validation(
                "operational crypto is outside MVP 0.1; use an isolated test harness".to_string(),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GatewaySigningConfig {
    pub policy: SigningPolicy,
    pub key_path: Option<PathBuf>,
    pub link_id: u8,
}

impl GatewaySigningConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        match self.policy {
            SigningPolicy::Observe => Ok(()),
            SigningPolicy::Audit | SigningPolicy::Enforce => {
                let key_path = self.key_path.as_ref().ok_or_else(|| {
                    ConfigError::Validation(format!(
                        "signing.key_path is required when signing.policy={}",
                        self.policy.as_str()
                    ))
                })?;
                validate_signing_key_path(key_path)
            }
        }
    }
}

impl Default for GatewaySigningConfig {
    fn default() -> Self {
        Self {
            policy: SigningPolicy::Observe,
            key_path: None,
            link_id: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SigningPolicy {
    Observe,
    Audit,
    Enforce,
}

impl SigningPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Observe => "observe",
            Self::Audit => "audit",
            Self::Enforce => "enforce",
        }
    }
}

fn validate_signing_key_path(path: &Path) -> Result<(), ConfigError> {
    if !path.is_absolute() {
        return Err(ConfigError::Validation(
            "signing.key_path must be an absolute local path".to_string(),
        ));
    }

    let symlink_metadata = fs::symlink_metadata(path).map_err(|source| {
        ConfigError::Validation(format!("signing.key_path could not be inspected: {source}"))
    })?;
    if symlink_metadata.file_type().is_symlink() {
        return Err(ConfigError::Validation(
            "signing.key_path must not point to a symbolic link".to_string(),
        ));
    }

    let metadata = fs::metadata(path).map_err(|source| {
        ConfigError::Validation(format!("signing.key_path could not be inspected: {source}"))
    })?;
    if !metadata.is_file() {
        return Err(ConfigError::Validation(
            "signing.key_path must point to a regular file".to_string(),
        ));
    }

    reject_key_inside_git_worktree(path)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};

        let mode = metadata.permissions().mode() & 0o777;
        if mode & 0o077 != 0 {
            return Err(ConfigError::Validation(
                "signing.key_path permissions must not allow group or world access".to_string(),
            ));
        }

        let owner_uid = metadata.uid();
        let process_uid = effective_uid();
        if owner_uid != process_uid {
            return Err(ConfigError::Validation(
                "signing.key_path owner must match the gateway process user".to_string(),
            ));
        }
    }

    Ok(())
}

fn reject_key_inside_git_worktree(path: &Path) -> Result<(), ConfigError> {
    let canonical_path = path.canonicalize().map_err(|source| {
        ConfigError::Validation(format!(
            "signing.key_path could not be canonicalized: {source}"
        ))
    })?;

    let current_dir = std::env::current_dir().map_err(|source| {
        ConfigError::Validation(format!(
            "current working directory could not be inspected: {source}"
        ))
    })?;
    let current_dir = current_dir.canonicalize().map_err(|source| {
        ConfigError::Validation(format!(
            "current working directory could not be canonicalized: {source}"
        ))
    })?;

    if let Some(worktree_root) = current_dir
        .ancestors()
        .find(|ancestor| ancestor.join(".git").exists())
    {
        if canonical_path.starts_with(worktree_root) {
            return Err(ConfigError::Validation(
                "signing.key_path must not be located inside the current Git worktree".to_string(),
            ));
        }
    }

    Ok(())
}

#[cfg(unix)]
fn effective_uid() -> u32 {
    extern "C" {
        fn geteuid() -> u32;
    }

    unsafe { geteuid() }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TransportConfig {
    pub mode: TransportMode,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            mode: TransportMode::Udp,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransportMode {
    Udp,
    Serial,
}

impl TransportMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Udp => "udp",
            Self::Serial => "serial",
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SerialConfig {
    pub port: PathBuf,
    pub baud_rate: u32,
    pub read_timeout_ms: u64,
    pub max_frame_size: usize,
}

impl SerialConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.port.as_os_str().is_empty() {
            return Err(ConfigError::Validation(
                "serial.port is required when transport.mode=serial".to_string(),
            ));
        }

        if self.baud_rate == 0 || self.baud_rate > 4_000_000 {
            return Err(ConfigError::Validation(
                "serial.baud_rate must be between 1 and 4000000".to_string(),
            ));
        }

        if self.read_timeout_ms == 0 || self.read_timeout_ms > 10_000 {
            return Err(ConfigError::Validation(
                "serial.read_timeout_ms must be between 1 and 10000 milliseconds".to_string(),
            ));
        }

        if self.max_frame_size < 8 || self.max_frame_size > 4096 {
            return Err(ConfigError::Validation(
                "serial.max_frame_size must be between 8 and 4096 bytes".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for SerialConfig {
    fn default() -> Self {
        Self {
            port: PathBuf::new(),
            baud_rate: 57_600,
            read_timeout_ms: 100,
            max_frame_size: 280,
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UdpConfig {
    pub listen_gcs: SocketAddr,
    pub listen_vehicle: SocketAddr,
    pub vehicle_addr: SocketAddr,
    pub gcs_addr: SocketAddr,
    pub read_timeout_ms: u64,
    pub max_datagram_size: usize,
}

impl Default for UdpConfig {
    fn default() -> Self {
        Self {
            listen_gcs: "127.0.0.1:14551".parse().expect("valid default socket"),
            listen_vehicle: "127.0.0.1:14550".parse().expect("valid default socket"),
            vehicle_addr: "127.0.0.1:14540".parse().expect("valid default socket"),
            gcs_addr: "127.0.0.1:14552".parse().expect("valid default socket"),
            read_timeout_ms: 100,
            max_datagram_size: 2048,
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SecurityConfig {
    pub certified_ips: Vec<IpAddr>,
    pub unknown_mode_policy: UnknownModePolicy,
    pub audit_only: bool,
    #[serde(default)]
    pub shadow_enforce: bool,
    pub block_arm_in_auto_mode: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            certified_ips: vec!["127.0.0.1".parse().expect("valid default IP")],
            unknown_mode_policy: UnknownModePolicy::Block,
            audit_only: false,
            shadow_enforce: false,
            block_arm_in_auto_mode: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UnknownModePolicy {
    Block,
    AuditOnly,
    Allow,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CryptoConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LoggingConfig {
    pub level: String,
    pub payload_logging: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            payload_logging: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub readonly_bind: Option<SocketAddr>,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            readonly_bind: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_valid_mvp_config() {
        let input = r#"
            [transport]
            mode = "udp"

            [udp]
            listen_gcs = "127.0.0.1:14551"
            listen_vehicle = "127.0.0.1:14550"
            vehicle_addr = "127.0.0.1:14540"
            gcs_addr = "127.0.0.1:14552"
            read_timeout_ms = 100
            max_datagram_size = 2048

            [security]
            certified_ips = ["127.0.0.1"]
            unknown_mode_policy = "block"
            audit_only = false
            shadow_enforce = false
            block_arm_in_auto_mode = true

            [signing]
            policy = "observe"
            link_id = 0

            [crypto]
            enabled = false

            [logging]
            level = "info"
            payload_logging = false

            [metrics]
            enabled = true
            readonly_bind = "127.0.0.1:14600"
        "#;

        let config = AppConfig::load_from_str(input).expect("valid config");

        assert_eq!(config.transport.mode, TransportMode::Udp);
        assert_eq!(
            config.security.unknown_mode_policy,
            UnknownModePolicy::Block
        );
        assert_eq!(config.signing.policy, SigningPolicy::Observe);
        assert!(!config.security.shadow_enforce);
        assert_eq!(
            config.metrics.readonly_bind,
            Some("127.0.0.1:14600".parse().expect("valid socket"))
        );
    }

    #[test]
    fn accepts_shadow_enforce_flag() {
        let mut config = AppConfig::default();
        config.security.shadow_enforce = true;

        config.validate().expect("shadow enforce is accepted");
        assert!(config.security.shadow_enforce);
    }

    #[test]
    fn accepts_serial_transport_config_for_virtual_lab() {
        let input = r#"
            [transport]
            mode = "serial"

            [udp]
            listen_gcs = "127.0.0.1:14551"
            listen_vehicle = "127.0.0.1:14550"
            vehicle_addr = "127.0.0.1:14540"
            gcs_addr = "127.0.0.1:14552"
            read_timeout_ms = 100
            max_datagram_size = 2048

            [serial]
            port = "/dev/pts/99"
            baud_rate = 57600
            read_timeout_ms = 100
            max_frame_size = 280

            [security]
            certified_ips = ["127.0.0.1"]
            unknown_mode_policy = "block"
            audit_only = false
            shadow_enforce = false
            block_arm_in_auto_mode = true

            [signing]
            policy = "observe"
            link_id = 0

            [crypto]
            enabled = false

            [logging]
            level = "info"
            payload_logging = false

            [metrics]
            enabled = true
        "#;

        let config = AppConfig::load_from_str(input).expect("serial config is accepted");

        assert_eq!(config.transport.mode, TransportMode::Serial);
        assert_eq!(config.serial.baud_rate, 57_600);
        assert_eq!(config.serial.max_frame_size, 280);
    }

    #[test]
    fn rejects_serial_transport_without_port() {
        let mut config = AppConfig::default();
        config.transport.mode = TransportMode::Serial;

        let err = config
            .validate()
            .expect_err("serial mode requires explicit port");

        assert!(err.to_string().contains("serial.port"));
    }

    #[test]
    fn rejects_allow_unknown_mode_for_critical_commands() {
        let mut config = AppConfig::default();
        config.security.unknown_mode_policy = UnknownModePolicy::Allow;

        let err = config.validate().expect_err("unsafe config must fail");

        assert!(err.to_string().contains("unknown_mode_policy=allow"));
    }

    #[test]
    fn rejects_missing_required_udp_fields() {
        let input = r#"
            [transport]
            mode = "udp"

            [udp]
            listen_gcs = "127.0.0.1:14551"
            listen_vehicle = "127.0.0.1:14550"
            vehicle_addr = "127.0.0.1:14540"
            gcs_addr = "127.0.0.1:14552"
            read_timeout_ms = 100

            [security]
            certified_ips = ["127.0.0.1"]
            unknown_mode_policy = "block"
            audit_only = false
            shadow_enforce = false
            block_arm_in_auto_mode = true

            [signing]
            policy = "observe"
            link_id = 0

            [crypto]
            enabled = false

            [logging]
            level = "info"
            payload_logging = false

            [metrics]
            enabled = true
            readonly_bind = "127.0.0.1:14600"
        "#;

        let err = AppConfig::load_from_str(input).expect_err("incomplete config must fail");

        assert!(err
            .to_string()
            .contains("missing field `max_datagram_size`"));
    }

    #[test]
    fn rejects_ambiguous_udp_bindings() {
        let mut config = AppConfig::default();
        config.udp.listen_vehicle = config.udp.listen_gcs;

        let err = config
            .validate()
            .expect_err("ambiguous UDP bindings must fail");

        assert!(err.to_string().contains("udp.listen_gcs"));
    }

    #[test]
    fn rejects_zero_udp_timeout() {
        let mut config = AppConfig::default();
        config.udp.read_timeout_ms = 0;

        let err = config.validate().expect_err("zero timeout must fail");

        assert!(err.to_string().contains("udp.read_timeout_ms"));
    }

    #[test]
    fn rejects_invalid_logging_filter() {
        let mut config = AppConfig::default();
        config.logging.level = "mavlink[broken".to_string();

        let err = config
            .validate()
            .expect_err("invalid tracing filter must fail");

        assert!(err.to_string().contains("logging.level"));
    }

    #[test]
    fn rejects_signing_audit_without_key_path() {
        let mut config = AppConfig::default();
        config.signing.policy = SigningPolicy::Audit;

        let err = config
            .validate()
            .expect_err("audit signing must require explicit key path");

        assert!(err.to_string().contains("signing.key_path"));
    }

    #[test]
    fn rejects_signing_enforce_without_key_path() {
        let mut config = AppConfig::default();
        config.signing.policy = SigningPolicy::Enforce;

        let err = config
            .validate()
            .expect_err("enforce signing must require explicit key path");

        assert!(err.to_string().contains("signing.key_path"));
        assert!(err.to_string().contains("policy=enforce"));
    }

    #[test]
    fn rejects_readonly_metrics_bind_on_non_loopback_address() {
        let mut config = AppConfig::default();
        config.metrics.readonly_bind = Some("0.0.0.0:14600".parse().expect("valid socket"));

        let err = config
            .validate()
            .expect_err("read-only endpoint must stay local in this phase");

        assert!(err.to_string().contains("metrics.readonly_bind"));
    }

    #[test]
    fn accepts_signing_audit_key_with_restrictive_permissions() {
        let temp = tempfile::NamedTempFile::new().expect("temp key file");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(0o600))
                .expect("set restrictive permissions");
        }

        let mut config = AppConfig::default();
        config.signing.policy = SigningPolicy::Audit;
        config.signing.key_path = Some(temp.path().to_path_buf());

        config.validate().expect("audit key path is accepted");
    }

    #[test]
    fn accepts_signing_enforce_key_with_restrictive_permissions() {
        let temp = tempfile::NamedTempFile::new().expect("temp key file");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(0o600))
                .expect("set restrictive permissions");
        }

        let mut config = AppConfig::default();
        config.signing.policy = SigningPolicy::Enforce;
        config.signing.key_path = Some(temp.path().to_path_buf());

        config.validate().expect("enforce key path is accepted");
    }

    #[test]
    fn rejects_signing_key_with_group_permissions() {
        let temp = tempfile::NamedTempFile::new().expect("temp key file");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(0o640))
                .expect("set group-readable permissions");
        }

        let mut config = AppConfig::default();
        config.signing.policy = SigningPolicy::Audit;
        config.signing.key_path = Some(temp.path().to_path_buf());

        #[cfg(unix)]
        assert!(config
            .validate()
            .expect_err("group-readable key must fail")
            .to_string()
            .contains("permissions"));
    }

    #[test]
    fn rejects_signing_key_inside_git_worktree() {
        let path = std::env::current_dir()
            .expect("current dir")
            .join("target/test-signing-key-not-secret.hex");
        std::fs::create_dir_all(path.parent().expect("path has parent")).expect("target dir");
        std::fs::write(
            &path,
            "102132435465768798a9bacbdcedfe0ff0e1d2c3b4a5968778695a4b3c2d1e0f",
        )
        .expect("write temp key");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
                .expect("set restrictive permissions");
        }

        let mut config = AppConfig::default();
        config.signing.policy = SigningPolicy::Audit;
        config.signing.key_path = Some(path.clone());

        let err = config
            .validate()
            .expect_err("key inside git worktree must fail");

        assert!(err.to_string().contains("Git worktree"));
        std::fs::remove_file(path).expect("remove temp key");
    }

    #[cfg(unix)]
    #[test]
    fn rejects_signing_key_symlink() {
        use std::os::unix::fs::{symlink, PermissionsExt};

        let dir = tempfile::tempdir().expect("temp dir");
        let key_path = dir.path().join("lab-signing.key");
        let symlink_path = dir.path().join("lab-signing-link.key");
        std::fs::write(
            &key_path,
            "102132435465768798a9bacbdcedfe0ff0e1d2c3b4a5968778695a4b3c2d1e0f",
        )
        .expect("write key");
        std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600))
            .expect("set restrictive permissions");
        symlink(&key_path, &symlink_path).expect("create symlink");

        let mut config = AppConfig::default();
        config.signing.policy = SigningPolicy::Enforce;
        config.signing.key_path = Some(symlink_path);

        assert!(config
            .validate()
            .expect_err("symlink key path must fail")
            .to_string()
            .contains("symbolic link"));
    }
}
