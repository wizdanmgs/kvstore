use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::signal;

use crate::command::Command;
use crate::store::Store;

// =========================================================
// TCP SERVER ENTRY POINT
// =========================================================
pub async fn run(
    addr: &str,
    store: Arc<Store>, // Shared application state
) -> anyhow::Result<()> {
    // Bind TCP listener to given address
    let listener = TcpListener::bind(addr).await?;
    println!("Server listening on {}", addr);

    loop {
        tokio::select! {
            // =====================================================
            // 1️⃣ ACCEPT NEW CONNECTION
            // =====================================================
            accept_result = listener.accept() => {
                let (socket, _) = accept_result?;

                // Clone Arc so each connection has shared access
                let store = store.clone();

                // =====================================================
                // 2️⃣ SPAWN ASYNC TASK FOR CLIENT
                // -----------------------------------------------------
                // Each client runs independently.
                // =====================================================
                tokio::spawn(async move {
                    // Split socket into reader & writer halves
                    let (reader, mut writer) = socket.into_split();
                    let mut reader = BufReader::new(reader);
                    let mut line = String::new();

                    loop {
                        // =============================================
                        // 3️⃣ READ CLIENT INPUT (line-based protocol)
                        // =============================================
                        let bytes_read = reader.read_line(&mut line).await;

                        // If connection closed or error -> exit loop
                        if bytes_read.is_err() || bytes_read.unwrap() == 0 {
                            break;
                        }

                        // =============================================
                        // 4️⃣ PARSE + EXECUTE COMMAND
                        // =============================================
                        let response = match Command::parse(&line) {
                            Ok(cmd) => cmd.execute(&store),
                            Err(e) => format!("ERR {}\n", e),
                        };

                        // =============================================
                        // 5️⃣ WRITE RESPONSE BACK TO CLIENT
                        // =============================================
                        if writer.write_all(response.as_bytes()).await.is_err() {
                            break;
                        }

                        // Clear buffer for next read
                        line.clear();
                    }
                });
            }

            // Shutdown signal
            _ = signal::ctrl_c() => {
                println!("\nReceived shutdown signal.");
                break;
            }
        }
    }
    println!("Shutting down gracefully...");
    Ok(())
}
