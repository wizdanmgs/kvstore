use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::store::Store;

// =========================================================
// Append a SET operation to WAL file
// =========================================================
pub fn append_set(key: &str, value: &str) -> anyhow::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("wal.log")?;

    // Simple line protocol
    // Format: SET key value\n
    writeln!(file, "SET {} {}", key, value)?;

    Ok(())
}

// =========================================================
// Replay WAL on startup to rebuild memory
// =========================================================
pub fn replay(store: &Store) -> anyhow::Result<()> {
    if !Path::new("wal.log").exists() {
        return Ok(());
    }

    let file = File::open("wal.log")?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() == 3 && parts[0] == "SET" {
            // Use set_internal instead of set to avoid rewriting to WAL during replay
            store.set_internal(parts[1].to_string(), parts[2].to_string());
        }
    }

    Ok(())
}
