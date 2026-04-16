use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Credentials {
    pub access_token: Option<String>,
}

pub fn credentials_file(config_dir: &Path) -> PathBuf {
    config_dir.join("credentials.json")
}

pub fn load_credentials(config_dir: &Path) -> Credentials {
    let path = credentials_file(config_dir);
    let Ok(raw) = fs::read_to_string(path) else {
        return Credentials::default();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

pub fn save_credentials(config_dir: &Path, creds: &Credentials) -> io::Result<()> {
    fs::create_dir_all(config_dir)?;
    let path = credentials_file(config_dir);
    let json = serde_json::to_string_pretty(creds)?;
    fs::write(path, json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credentials_file_uses_config_dir() {
        let base = PathBuf::from("C:/tmp/saya-config");
        let file = credentials_file(&base);
        assert!(file.to_string_lossy().contains("saya-config"));
        assert!(file.to_string_lossy().ends_with("credentials.json"));
    }

    #[test]
    fn save_credentials_writes_under_config_dir() {
        let unique = format!(
            "saya-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time moved backwards")
                .as_nanos()
        );
        let base = std::env::temp_dir().join(unique);
        let creds = Credentials {
            access_token: Some("tok".to_string()),
        };
        save_credentials(&base, &creds).expect("save should succeed");
        let file = credentials_file(&base);
        assert!(file.exists());
        let loaded = load_credentials(&base);
        assert_eq!(loaded.access_token.as_deref(), Some("tok"));
        std::fs::remove_file(file).ok();
        std::fs::remove_dir_all(base).ok();
    }
}
