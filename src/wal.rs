use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::store::Store;

// =========================================================
// Append a SET operation to WAL file
// =========================================================
pub fn append_set(key: &str, value: &str, ttl: Option<u64>) -> anyhow::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("wal.log")?;

    match ttl {
        Some(t) => writeln!(file, "SET {} {} {}", key, value, t)?,
        None => writeln!(file, "SET {} {}", key, value)?,
    }

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

        // Use set_internal instead of set to avoid rewriting to WAL during replay
        match parts.as_slice() {
            ["SET", key, value] => {
                store.set_internal(key.to_string(), value.to_string(), None);
            }
            ["SET", key, value, ttl] => {
                let ttl = ttl.parse::<u64>().ok();
                store.set_internal(key.to_string(), value.to_string(), ttl);
            }
            _ => {}
        }
    }

    Ok(())
}
