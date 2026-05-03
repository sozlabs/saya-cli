use clap::{Parser, Subcommand};
use saya_cli::commands::{run_chat, run_health, run_version};
use saya_cli::config::{CliConfig, ConfigFlags, OutputFormat};
use saya_cli::credentials::{load_credentials, save_credentials};
use saya_cli::debug::{debug_log, should_debug};
use saya_cli::terminal_guard::install_panic_terminal_hook;
use saya_cli::transport::http::{build_http_client, HttpTransport};
use std::io::{self, BufRead};

#[derive(Parser)]
#[command(name = "saya")]
#[command(about = "CLI client for soz-saya (thin transport layer)", long_about = None)]
struct Cli {
    /// Base URL of soz-saya, e.g. http://127.0.0.1:3010
    #[arg(long)]
    base_url: Option<String>,

    /// Timeout in milliseconds
    #[arg(long)]
    timeout_ms: Option<u64>,

    /// Output format: text or json
    #[arg(long, value_parser = ["text", "json"])]
    output: Option<String>,

    /// Disable interactive prompts
    #[arg(long, default_value_t = false)]
    non_interactive: bool,

    /// Enable debug transport logs with secret redaction
    #[arg(long, default_value_t = false)]
    debug: bool,

    #[arg(long)]
    conversation_id: Option<String>,

    #[arg(long)]
    session_id: Option<String>,

    #[arg(long)]
    tenant_id: Option<String>,

    #[arg(long)]
    actor_id: Option<String>,

    #[arg(long)]
    channel_id: Option<String>,

    /// Max extra attempts to reopen the SSE stream after I/O failure (each attempt is a new POST).
    #[arg(long)]
    stream_max_retries: Option<u32>,

    #[arg(long)]
    stream_retry_initial_ms: Option<u64>,

    #[arg(long)]
    stream_retry_max_ms: Option<u64>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Version,
    Health,

    /// Store the Bearer access token used by `saya chat`
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },

    Chat {
        #[arg(long)]
        message: String,
        /// Allow allowlisted restricted (write-like) tools without prompting (use with care in scripts).
        #[arg(long, default_value_t = false)]
        allow_restricted_tools: bool,
    },
}

#[derive(Subcommand)]
enum AuthCommands {
    /// Write access token to credentials file (use `--token` or pipe a line on stdin)
    Set {
        /// Token string (omit to read one line from stdin; safer than shell history)
        #[arg(long, value_name = "TOKEN")]
        token: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    install_panic_terminal_hook();
    let cli = Cli::parse();
    let output_format = match cli.output.as_deref() {
        Some("json") => Some(OutputFormat::Json),
        Some("text") => Some(OutputFormat::Text),
        _ => None,
    };
    let config = CliConfig::load(ConfigFlags {
        base_url: cli.base_url,
        timeout_ms: cli.timeout_ms,
        output_format,
        non_interactive: cli.non_interactive,
        debug: cli.debug,
        conversation_id: cli.conversation_id,
        session_id: cli.session_id,
        tenant_id: cli.tenant_id,
        actor_id: cli.actor_id,
        channel_id: cli.channel_id,
        stream_max_retries: cli.stream_max_retries,
        stream_retry_initial_ms: cli.stream_retry_initial_ms,
        stream_retry_max_ms: cli.stream_retry_max_ms,
    });
    let debug_enabled = should_debug(config.debug);
    let mut credentials = load_credentials(&config.config_dir);

    let client = match build_http_client() {
        Ok(c) => c,
        Err(err) => {
            eprintln!("failed to build HTTP client: {err}");
            std::process::exit(1);
        }
    };
    let transport = HttpTransport::new(client);
    debug_log(
        debug_enabled,
        &format!(
            "startup base_url={} timeout_ms={} non_interactive={} output={:?}",
            config.base_url,
            config.timeout.as_millis(),
            config.non_interactive,
            config.output_format
        ),
    );
    match cli.command {
        Commands::Version => {
            println!("{}", run_version(&config));
        }
        Commands::Health => match run_health(&transport, &config).await {
            Ok(output) => println!("{}", output),
            Err(err) => {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        },
        Commands::Auth { command } => match command {
            AuthCommands::Set { token } => {
                let raw = match token {
                    Some(value) => value,
                    None => {
                        let mut line = String::new();
                        if let Err(err) = io::stdin().lock().read_line(&mut line) {
                            eprintln!("failed to read token from stdin: {err}");
                            std::process::exit(1);
                        }
                        line
                    }
                };
                let trimmed = raw.trim();
                if trimmed.is_empty() {
                    eprintln!("error: token is empty (use --token or pipe one line on stdin)");
                    std::process::exit(1);
                }
                credentials.access_token = Some(trimmed.to_string());
                debug_log(
                    debug_enabled,
                    "auth set: access token written to credentials file",
                );
                if let Err(err) = save_credentials(&config.config_dir, &credentials) {
                    eprintln!("failed to persist credentials: {err}");
                    std::process::exit(1);
                }
                if matches!(config.output_format, OutputFormat::Text) {
                    println!("access token saved");
                } else {
                    println!(
                        "{}",
                        serde_json::json!({"command":"auth set","ok":true,"output":"access token saved"})
                    );
                }
                return;
            }
        },
        Commands::Chat {
            message,
            allow_restricted_tools,
        } => match run_chat(
            &transport,
            &config,
            &message,
            &mut credentials,
            allow_restricted_tools,
        )
        .await
        {
            Ok(output) => println!("{}", output),
            Err(err) => {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        },
    }
    if let Err(err) = save_credentials(&config.config_dir, &credentials) {
        eprintln!("failed to persist credentials: {}", err);
    }
}
