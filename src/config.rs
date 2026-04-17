use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FileConfig {
    pub base_url: Option<String>,
    pub timeout_ms: Option<u64>,
    pub output_format: Option<OutputFormat>,
    pub non_interactive: Option<bool>,
    pub debug: Option<bool>,
    pub conversation_id: Option<String>,
    pub session_id: Option<String>,
    pub tenant_id: Option<String>,
    pub actor_id: Option<String>,
    pub channel_id: Option<String>,
    /// Max full-stream retry attempts after transport failure (not counting first try).
    pub stream_max_retries: Option<u32>,
    pub stream_retry_initial_ms: Option<u64>,
    pub stream_retry_max_ms: Option<u64>,
}

#[derive(Clone, Debug, Default)]
struct EnvConfig {
    base_url: Option<String>,
    timeout_ms: Option<u64>,
    output_format: Option<OutputFormat>,
    non_interactive: Option<bool>,
    debug: Option<bool>,
    conversation_id: Option<String>,
    session_id: Option<String>,
    tenant_id: Option<String>,
    actor_id: Option<String>,
    channel_id: Option<String>,
    stream_max_retries: Option<u32>,
    stream_retry_initial_ms: Option<u64>,
    stream_retry_max_ms: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct ConfigFlags {
    pub base_url: Option<String>,
    pub timeout_ms: Option<u64>,
    pub output_format: Option<OutputFormat>,
    pub non_interactive: bool,
    pub debug: bool,
    pub conversation_id: Option<String>,
    pub session_id: Option<String>,
    pub tenant_id: Option<String>,
    pub actor_id: Option<String>,
    pub channel_id: Option<String>,
    pub stream_max_retries: Option<u32>,
    pub stream_retry_initial_ms: Option<u64>,
    pub stream_retry_max_ms: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct CliConfig {
    pub base_url: String,
    pub timeout: Duration,
    pub output_format: OutputFormat,
    pub non_interactive: bool,
    pub debug: bool,
    pub config_dir: PathBuf,
    pub conversation_id: Option<String>,
    pub session_id: String,
    pub tenant_id: String,
    pub actor_id: String,
    pub channel_id: String,
    /// Full-stream retries after I/O or incomplete SSE (each retry is a new POST).
    pub stream_max_retries: u32,
    pub stream_retry_initial_ms: u64,
    pub stream_retry_max_ms: u64,
}

fn parse_output(value: &str) -> Option<OutputFormat> {
    match value.to_ascii_lowercase().as_str() {
        "text" => Some(OutputFormat::Text),
        "json" => Some(OutputFormat::Json),
        _ => None,
    }
}

pub fn resolve_config_dir() -> PathBuf {
    if let Ok(override_dir) = env::var("SAYA_CONFIG_DIR") {
        return PathBuf::from(override_dir);
    }
    if let Some(project_dirs) = ProjectDirs::from("", "", "saya") {
        return project_dirs.config_dir().to_path_buf();
    }
    PathBuf::from(".saya")
}

pub fn resolve_config_file(dir: &std::path::Path) -> PathBuf {
    dir.join("config.json")
}

fn load_file_config(path: &std::path::Path) -> Option<FileConfig> {
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str::<FileConfig>(&raw).ok()
}

impl CliConfig {
    fn from_sources(
        flags: ConfigFlags,
        env_config: EnvConfig,
        file_config: Option<FileConfig>,
        config_dir: PathBuf,
    ) -> Self {
        let base_url = flags
            .base_url
            .or(env_config.base_url)
            .or_else(|| file_config.as_ref().and_then(|c| c.base_url.clone()));
        let base_url = match base_url {
            Some(value) => value,
            None => "http://127.0.0.1:3010".to_string(),
        };
        let timeout_ms = flags
            .timeout_ms
            .or(env_config.timeout_ms)
            .or_else(|| file_config.as_ref().and_then(|c| c.timeout_ms));
        let timeout_ms = timeout_ms.map_or(30_000, |value| value);
        let output_format = flags
            .output_format
            .or(env_config.output_format)
            .or_else(|| file_config.as_ref().and_then(|c| c.output_format));
        let output_format = match output_format {
            Some(value) => value,
            None => OutputFormat::Text,
        };
        let non_interactive = if flags.non_interactive {
            true
        } else {
            let value = env_config
                .non_interactive
                .or_else(|| file_config.as_ref().and_then(|c| c.non_interactive));
            value.is_some_and(|v| v)
        };
        let debug = if flags.debug {
            true
        } else {
            let value = env_config
                .debug
                .or_else(|| file_config.as_ref().and_then(|c| c.debug));
            value.is_some_and(|v| v)
        };
        Self {
            base_url,
            timeout: Duration::from_millis(timeout_ms),
            output_format,
            non_interactive,
            debug,
            config_dir,
            conversation_id: flags
                .conversation_id
                .or(env_config.conversation_id)
                .or_else(|| file_config.as_ref().and_then(|c| c.conversation_id.clone())),
            session_id: flags
                .session_id
                .or(env_config.session_id)
                .or_else(|| file_config.as_ref().and_then(|c| c.session_id.clone()))
                .map_or("terminal-session".to_string(), |value| value),
            tenant_id: flags
                .tenant_id
                .or(env_config.tenant_id)
                .or_else(|| file_config.as_ref().and_then(|c| c.tenant_id.clone()))
                .map_or("terminal-tenant".to_string(), |value| value),
            actor_id: flags
                .actor_id
                .or(env_config.actor_id)
                .or_else(|| file_config.as_ref().and_then(|c| c.actor_id.clone()))
                .map_or("terminal-user".to_string(), |value| value),
            channel_id: flags
                .channel_id
                .or(env_config.channel_id)
                .or_else(|| file_config.as_ref().and_then(|c| c.channel_id.clone()))
                .map_or("terminal".to_string(), |value| value),
            stream_max_retries: flags
                .stream_max_retries
                .or(env_config.stream_max_retries)
                .or_else(|| file_config.as_ref().and_then(|c| c.stream_max_retries))
                .unwrap_or(3),
            stream_retry_initial_ms: flags
                .stream_retry_initial_ms
                .or(env_config.stream_retry_initial_ms)
                .or_else(|| file_config.as_ref().and_then(|c| c.stream_retry_initial_ms))
                .unwrap_or(500),
            stream_retry_max_ms: flags
                .stream_retry_max_ms
                .or(env_config.stream_retry_max_ms)
                .or_else(|| file_config.as_ref().and_then(|c| c.stream_retry_max_ms))
                .unwrap_or(8000),
        }
    }

    pub fn load(flags: ConfigFlags) -> Self {
        let config_dir = resolve_config_dir();
        let file_config = load_file_config(&resolve_config_file(&config_dir));

        let env_config = EnvConfig {
            base_url: env::var("SAYA_BASE_URL").ok(),
            timeout_ms: env::var("SAYA_TIMEOUT_MS")
                .ok()
                .and_then(|value| value.parse::<u64>().ok()),
            output_format: env::var("SAYA_OUTPUT_FORMAT")
                .ok()
                .and_then(|value| parse_output(&value)),
            non_interactive: env::var("SAYA_NON_INTERACTIVE")
                .ok()
                .map(|value| value == "1" || value.eq_ignore_ascii_case("true")),
            debug: env::var("SAYA_DEBUG")
                .ok()
                .map(|value| value == "1" || value.eq_ignore_ascii_case("true")),
            conversation_id: env::var("SAYA_CONVERSATION_ID").ok(),
            session_id: env::var("SAYA_SESSION_ID").ok(),
            tenant_id: env::var("SAYA_TENANT_ID").ok(),
            actor_id: env::var("SAYA_ACTOR_ID").ok(),
            channel_id: env::var("SAYA_CHANNEL_ID").ok(),
            stream_max_retries: env::var("SAYA_STREAM_MAX_RETRIES")
                .ok()
                .and_then(|value| value.parse::<u32>().ok()),
            stream_retry_initial_ms: env::var("SAYA_STREAM_RETRY_INITIAL_MS")
                .ok()
                .and_then(|value| value.parse::<u64>().ok()),
            stream_retry_max_ms: env::var("SAYA_STREAM_RETRY_MAX_MS")
                .ok()
                .and_then(|value| value.parse::<u64>().ok()),
        };
        Self::from_sources(flags, env_config, file_config, config_dir)
    }
}

#[cfg(test)]
impl CliConfig {
    fn from_test_sources(
        flags: ConfigFlags,
        env_config: EnvConfig,
        file_config: Option<FileConfig>,
    ) -> Self {
        Self::from_sources(flags, env_config, file_config, PathBuf::from(".saya-test"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_flags() -> ConfigFlags {
        ConfigFlags {
            base_url: None,
            timeout_ms: None,
            output_format: None,
            non_interactive: false,
            debug: false,
            conversation_id: None,
            session_id: None,
            tenant_id: None,
            actor_id: None,
            channel_id: None,
            stream_max_retries: None,
            stream_retry_initial_ms: None,
            stream_retry_max_ms: None,
        }
    }

    #[test]
    fn flags_override_defaults() {
        let cfg = CliConfig::from_test_sources(
            ConfigFlags {
                base_url: Some("http://localhost:9999".to_string()),
                timeout_ms: Some(1_500),
                output_format: Some(OutputFormat::Json),
                non_interactive: true,
                debug: true,
                conversation_id: Some("cid-flags".to_string()),
                session_id: Some("s-flags".to_string()),
                tenant_id: Some("t-flags".to_string()),
                actor_id: Some("u-flags".to_string()),
                channel_id: Some("terminal".to_string()),
                stream_max_retries: None,
                stream_retry_initial_ms: None,
                stream_retry_max_ms: None,
            },
            EnvConfig::default(),
            None,
        );
        assert_eq!(cfg.base_url, "http://localhost:9999");
        assert_eq!(cfg.timeout.as_millis(), 1_500);
        assert_eq!(cfg.output_format, OutputFormat::Json);
        assert!(cfg.non_interactive);
        assert!(cfg.debug);
        assert_eq!(cfg.conversation_id.as_deref(), Some("cid-flags"));
        assert_eq!(cfg.session_id, "s-flags");
        assert_eq!(cfg.tenant_id, "t-flags");
        assert_eq!(cfg.actor_id, "u-flags");
        assert_eq!(cfg.channel_id, "terminal");
        assert_eq!(cfg.stream_max_retries, 3);
        assert_eq!(cfg.stream_retry_initial_ms, 500);
        assert_eq!(cfg.stream_retry_max_ms, 8000);
    }

    #[test]
    fn env_overrides_file_and_defaults() {
        let cfg = CliConfig::from_test_sources(
            empty_flags(),
            EnvConfig {
                base_url: Some("http://env.example".to_string()),
                timeout_ms: Some(1234),
                output_format: Some(OutputFormat::Json),
                non_interactive: Some(true),
                debug: Some(true),
                conversation_id: Some("cid-1".to_string()),
                session_id: Some("s1".to_string()),
                tenant_id: Some("t1".to_string()),
                actor_id: Some("u1".to_string()),
                channel_id: Some("terminal".to_string()),
                stream_max_retries: None,
                stream_retry_initial_ms: None,
                stream_retry_max_ms: None,
            },
            Some(FileConfig {
                base_url: Some("http://file.example".to_string()),
                timeout_ms: Some(999),
                output_format: Some(OutputFormat::Text),
                non_interactive: Some(false),
                debug: Some(false),
                conversation_id: Some("cid-file".to_string()),
                session_id: Some("s-file".to_string()),
                tenant_id: Some("t-file".to_string()),
                actor_id: Some("u-file".to_string()),
                channel_id: Some("web".to_string()),
                stream_max_retries: None,
                stream_retry_initial_ms: None,
                stream_retry_max_ms: None,
            }),
        );
        assert_eq!(cfg.base_url, "http://env.example");
        assert_eq!(cfg.timeout.as_millis(), 1234);
        assert_eq!(cfg.output_format, OutputFormat::Json);
        assert!(cfg.non_interactive);
        assert!(cfg.debug);
        assert_eq!(cfg.conversation_id.as_deref(), Some("cid-1"));
        assert_eq!(cfg.session_id, "s1");
        assert_eq!(cfg.tenant_id, "t1");
        assert_eq!(cfg.actor_id, "u1");
        assert_eq!(cfg.channel_id, "terminal");
    }

    #[test]
    fn file_used_when_flags_and_env_missing() {
        let cfg = CliConfig::from_test_sources(
            empty_flags(),
            EnvConfig::default(),
            Some(FileConfig {
                base_url: Some("http://file-only.example".to_string()),
                timeout_ms: Some(2222),
                output_format: Some(OutputFormat::Json),
                non_interactive: Some(true),
                debug: Some(true),
                conversation_id: Some("cid-file-only".to_string()),
                session_id: Some("s-file-only".to_string()),
                tenant_id: Some("t-file-only".to_string()),
                actor_id: Some("u-file-only".to_string()),
                channel_id: Some("terminal".to_string()),
                stream_max_retries: None,
                stream_retry_initial_ms: None,
                stream_retry_max_ms: None,
            }),
        );
        assert_eq!(cfg.base_url, "http://file-only.example");
        assert_eq!(cfg.timeout.as_millis(), 2222);
        assert_eq!(cfg.output_format, OutputFormat::Json);
        assert!(cfg.non_interactive);
        assert!(cfg.debug);
        assert_eq!(cfg.conversation_id.as_deref(), Some("cid-file-only"));
        assert_eq!(cfg.session_id, "s-file-only");
        assert_eq!(cfg.tenant_id, "t-file-only");
        assert_eq!(cfg.actor_id, "u-file-only");
        assert_eq!(cfg.channel_id, "terminal");
    }
}
