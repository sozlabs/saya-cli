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
}

#[derive(Clone, Debug, Default)]
struct EnvConfig {
    base_url: Option<String>,
    timeout_ms: Option<u64>,
    output_format: Option<OutputFormat>,
    non_interactive: Option<bool>,
    debug: Option<bool>,
}

#[derive(Clone, Debug)]
pub struct ConfigFlags {
    pub base_url: Option<String>,
    pub timeout_ms: Option<u64>,
    pub output_format: Option<OutputFormat>,
    pub non_interactive: bool,
    pub debug: bool,
}

#[derive(Clone, Debug)]
pub struct CliConfig {
    pub base_url: String,
    pub timeout: Duration,
    pub output_format: OutputFormat,
    pub non_interactive: bool,
    pub debug: bool,
    pub config_dir: PathBuf,
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
            .or_else(|| file_config.as_ref().and_then(|c| c.base_url.clone()))
            .unwrap_or_else(|| "http://127.0.0.1:3010".to_string());
        let timeout_ms = flags
            .timeout_ms
            .or(env_config.timeout_ms)
            .or_else(|| file_config.as_ref().and_then(|c| c.timeout_ms))
            .unwrap_or(30_000);
        let output_format = flags
            .output_format
            .or(env_config.output_format)
            .or_else(|| file_config.as_ref().and_then(|c| c.output_format))
            .unwrap_or(OutputFormat::Text);
        let non_interactive = if flags.non_interactive {
            true
        } else {
            env_config
                .non_interactive
                .or_else(|| file_config.as_ref().and_then(|c| c.non_interactive))
                .unwrap_or(false)
        };
        let debug = if flags.debug {
            true
        } else {
            env_config
                .debug
                .or_else(|| file_config.as_ref().and_then(|c| c.debug))
                .unwrap_or(false)
        };
        Self {
            base_url,
            timeout: Duration::from_millis(timeout_ms),
            output_format,
            non_interactive,
            debug,
            config_dir,
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
            },
            EnvConfig::default(),
            None,
        );
        assert_eq!(cfg.base_url, "http://localhost:9999");
        assert_eq!(cfg.timeout.as_millis(), 1_500);
        assert_eq!(cfg.output_format, OutputFormat::Json);
        assert!(cfg.non_interactive);
        assert!(cfg.debug);
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
            },
            Some(FileConfig {
                base_url: Some("http://file.example".to_string()),
                timeout_ms: Some(999),
                output_format: Some(OutputFormat::Text),
                non_interactive: Some(false),
                debug: Some(false),
            }),
        );
        assert_eq!(cfg.base_url, "http://env.example");
        assert_eq!(cfg.timeout.as_millis(), 1234);
        assert_eq!(cfg.output_format, OutputFormat::Json);
        assert!(cfg.non_interactive);
        assert!(cfg.debug);
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
            }),
        );
        assert_eq!(cfg.base_url, "http://file-only.example");
        assert_eq!(cfg.timeout.as_millis(), 2222);
        assert_eq!(cfg.output_format, OutputFormat::Json);
        assert!(cfg.non_interactive);
        assert!(cfg.debug);
    }
}
