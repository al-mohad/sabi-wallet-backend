use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Runs database migrations
    Migrate,
    /// Initializes the default admin user if one doesn't exist
    InitAdmin,
    /// Generates a new Nostr keypair and prints it
    GenerateNostrKeys,
    /// Starts the Sabi Wallet backend API server (default if no subcommand)
    Serve,
}

pub async fn run_cli_command(command: Commands) -> Result<()> {
    match command {
        Commands::Migrate => {
            info!("Running database migrations...");
            // TODO: Call actual migration logic
            info!("Migrations complete (placeholder).");
        }
        Commands::InitAdmin => {
            info!("Initializing default admin user...");
            // TODO: Call logic to ensure default admin user exists
            info!("Default admin user initialized (placeholder).");
        }
        Commands::GenerateNostrKeys => {
            info!("Generating a new Nostr keypair...");
            // TODO: Implement actual key generation and print securely
            let keys = nostr_sdk::nostr::Keys::generate();
            info!("New Nostr Public Key (npub): {}", keys.public_key().to_bech32()?);
            info!("New Nostr Secret Key (nsec): {}", keys.secret_key()?.to_secret_key().to_bech32()?);
        }
        Commands::Serve => {
            info!("Starting API server...");
            // This case is handled by `main` if no subcommand is given.
            // This `run_cli_command` will probably not be called for `Serve`.
        }
    }
    Ok(())
}
