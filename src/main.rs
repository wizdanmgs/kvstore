// Declare project modules
mod command;
mod persistence;
mod server;
mod store;
mod wal;

use clap::Parser;
use std::sync::Arc;
use tokio::time::{Duration, interval};

use store::Store;

// =====================================================
// CLI Definition
// =====================================================
#[derive(Parser, Debug)]
#[command(
    name = "kvstore",
    version,
    about = "A simplified Redis-like key-value store"
)]
struct Cli {
    /// Address to bind the TCP server
    #[arg(short, long, default_value = "127.0.0.1:6380")]
    addr: String,

    /// Database file path
    #[arg(short, long, default_value = "db.bin")]
    db: String,
}

// =====================================================
// Entry Point
// =====================================================
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // =========================================================
    // PARSE CLI arguments
    // ---------------------------------------------------------
    // Get address and DB file path from CLI
    // =========================================================
    let cli = Cli::parse();

    // =========================================================
    // LOAD DATABASE FROM DISK (if exists)
    // ---------------------------------------------------------
    // Try to load persisted database from "db.bin".
    // If file does not exist or fails to load,
    // fallback to a new empty Store.
    // =========================================================
    println!("Using database file {}", cli.db);

    let store = persistence::load(&cli.db).unwrap_or_else(|_| {
        println!("No existing DB found. Starting fresh.");
        Store::new(1000)
    });

    // =========================================================
    // WRAP STORE IN Arc<>
    // ---------------------------------------------------------
    // Arc -> allows multiple threads/tasks to share ownership
    // =========================================================
    let shared_store = Arc::new(store);

    // =========================================================
    // REPLAY WAL TO REBUILD DB
    // ---------------------------------------------------------
    // Replay WAL on startup to rebuild memory.
    // =========================================================
    wal::replay(&shared_store)?;

    // =========================================================
    // SET EXPIRATION WORKER
    // ---------------------------------------------------------
    // Clear expired keys every 5 seconds.
    // =========================================================
    {
        let store = shared_store.clone();
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(5));

            loop {
                ticker.tick().await;
                store.cleanup_expired();
            }
        });
    }

    // =========================================================
    // START TCP SERVER
    // ---------------------------------------------------------
    // Get port from argument or env.
    // Pass shared store into server so all connections
    // operate on the same in-memory database.
    // =========================================================
    server::run(&cli.addr, shared_store).await?;

    println!("Server exited cleanly.");

    Ok(())
}
