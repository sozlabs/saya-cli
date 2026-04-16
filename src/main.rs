use clap::{Parser, Subcommand};
use saya_cli::commands::{run_chat, run_health, run_version};
use saya_cli::transport::http::HttpTransport;

#[derive(Parser)]
#[command(name = "saya")]
#[command(about = "CLI client for soz-saya (thin transport layer)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {

    Version,

    Health {
        /// Base URL of soz-saya, e.g. http://127.0.0.1:3000
        #[arg(long, default_value = "http://127.0.0.1:3000")]
        base_url: String,
    },

    Chat {
        /// Base URL of soz-saya, e.g. http://127.0.0.1:3000
        #[arg(long, default_value = "http://127.0.0.1:3000")]
        base_url: String,

        #[arg(long)]
        message: String,
    },
}

fn main() {
    let cli = Cli::parse();
    let transport = HttpTransport;
    match cli.command {
        Commands::Version => {
            println!("{}", run_version());
        }
        Commands::Health { base_url } => {
            println!("{}", run_health(&transport, &base_url));
        }
        Commands::Chat { base_url, message } => {
            println!("{}", run_chat(&transport, &base_url, &message));
        }
    }
}
