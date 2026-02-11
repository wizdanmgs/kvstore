// Declare project modules
mod command;
mod persistence;
mod server;
mod store;

use std::sync::{Arc, Mutex};
use store::Store;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // =========================================================
    // 1️⃣  LOAD DATABASE FROM DISK (if exists)
    // ---------------------------------------------------------
    // Try to load persisted database from "db.bin".
    // If file does not exist or fails to load,
    // fallback to a new empty Store.
    // =========================================================
    let store = persistence::load("db.bin").unwrap_or_else(|_| {
        println!("No existing DB found. Starting fresh.");
        Store::new()
    });

    // =========================================================
    // 2️⃣  WRAP STORE IN Arc<Mutex<>>
    // ---------------------------------------------------------
    // Arc   -> allows multiple threads/tasks to share ownership
    // Mutex -> ensures safe mutable access from multiple clients
    // =========================================================
    let shared_store = Arc::new(Mutex::new(store));

    // =========================================================
    // 3️⃣  START TCP SERVER
    // ---------------------------------------------------------
    // Pass shared store into server so all connections
    // operate on the same in-memory database.
    // =========================================================
    server::run("127.0.0.1:6380", shared_store).await?;

    Ok(())
}
