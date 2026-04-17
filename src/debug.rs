use std::env;

pub fn should_debug(cli_debug: bool) -> bool {
    if cli_debug {
        return true;
    }
    let rust_log = env::var("RUST_LOG")
        .ok()
        .map_or(String::new(), |value| value);
    rust_log.to_ascii_lowercase().contains("debug")
}

pub fn redact_secrets(input: &str) -> String {
    input
        .lines()
        .map(|line| {
            let lower = line.to_ascii_lowercase();
            if lower.starts_with("authorization:") {
                return "Authorization: [REDACTED]".to_string();
            }
            if lower.starts_with("cookie:") {
                return "Cookie: [REDACTED]".to_string();
            }
            let mut sanitized = line.to_string();
            if let Some(index) = lower.find("refresh_token=") {
                sanitized = format!("{}refresh_token=[REDACTED]", &line[..index]);
            }
            sanitized.replace("Bearer ", "Bearer [REDACTED]")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn debug_log(enabled: bool, message: &str) {
    if !enabled {
        return;
    }
    eprintln!("[debug] {}", redact_secrets(message));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_common_secrets() {
        let raw = "Authorization: Bearer abc123\ncookie: sid=1\nrefresh_token=xyz\nsafe=value";
        let clean = redact_secrets(raw);
        assert!(!clean.contains("abc123"));
        assert!(!clean.contains("sid=1"));
        assert!(!clean.contains("xyz"));
        assert!(clean.contains("safe=value"));
    }
}
