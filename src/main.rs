// Declare project modules
mod command;
mod persistence;
mod server;
mod store;
mod wal;

use std::sync::Arc;
use tokio::time::{Duration, interval};

use store::Store;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // =========================================================
    // LOAD DATABASE FROM DISK (if exists)
    // ---------------------------------------------------------
    // Try to load persisted database from "db.bin".
    // If file does not exist or fails to load,
    // fallback to a new empty Store.
    // =========================================================
    let store = persistence::load("db.bin").unwrap_or_else(|_| {
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
    // Pass shared store into server so all connections
    // operate on the same in-memory database.
    // =========================================================
    server::run("127.0.0.1:6380", shared_store).await?;

    println!("Server exited cleanly.");

    Ok(())
}
