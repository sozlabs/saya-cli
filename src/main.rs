use clap::{Parser, Subcommand};
use saya_cli::commands::{run_chat, run_health, run_version};
use saya_cli::config::{CliConfig, ConfigFlags, OutputFormat};
use saya_cli::credentials::load_credentials;
use saya_cli::debug::{debug_log, should_debug};
use saya_cli::transport::http::HttpTransport;

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

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Version,
    Health,

    Chat {
        #[arg(long)]
        message: String,
    },
}

fn main() {
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
    });
    let debug_enabled = should_debug(config.debug);
    let credentials = load_credentials(&config.config_dir);

    let transport = HttpTransport;
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
        Commands::Health => match run_health(&transport, &config) {
            Ok(output) => println!("{}", output),
            Err(err) => {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        },
        Commands::Chat { message } => match run_chat(&transport, &config, &message, &credentials) {
            Ok(output) => println!("{}", output),
            Err(err) => {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        },
    }
}
