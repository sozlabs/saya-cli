use clap::{Parser, Subcommand};

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
        #[arg(long)]
        message: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Version => {
            println!("saya-cli {}", env!("CARGO_PKG_VERSION"));
        }
        Commands::Health { base_url } => {
            println!("health stub: would GET {base_url}/health");
        }
        Commands::Chat { message } => {
            println!("chat stub: {message}");
        }
    }
}
