use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::signal;

use crate::command::Command;
use crate::resp;
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
                    let (mut reader, mut writer) = socket.into_split();

                    loop {
                        // =============================================
                        // 4️⃣ PARSE + EXECUTE COMMAND
                        // =============================================
                        let parsed = resp::parse(&mut reader).await;

                        let response = match parsed {
                            Ok(resp::RespValue::Array(items)) => {
                                let args: Vec<String> = items.into_iter()
                                    .filter_map(|v| {
                                        if let resp::RespValue::BulkString(s) = v {
                                            Some(s)
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();

                                match Command::from_vec(args) {
                                    Ok(cmd) => cmd.execute(&store),
                                    Err(_) => resp::encode_error("ERR invalid request"),
                                }
                            }
                            Err(_) => break,
                            _ => break,
                        };

                        // =============================================
                        // 5️⃣ WRITE RESPONSE BACK TO CLIENT
                        // =============================================
                        if writer.write_all(response.as_bytes()).await.is_err() {
                            break;
                        }
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
