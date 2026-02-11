use crate::store::Store;
use std::fs::File;
use std::io::{Read, Write};

// =========================================================
// SAVE STORE TO DISK (SNAPSHOT)
// =========================================================
pub fn save(store: &Store, path: &str) -> anyhow::Result<()> {
    // Serialize store into compact binary (MessagePack)
    let encoded = rmp_serde::to_vec(store)?;

    // Create or overwrite file
    let mut file = File::create(path)?;

    // Write binary data to disk
    file.write_all(&encoded)?;

    Ok(())
}

// =========================================================
// LOAD STORE FROM DISK
// =========================================================
pub fn load(path: &str) -> anyhow::Result<Store> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();

    // Read entire file into memory
    file.read_to_end(&mut buffer)?;

    // Deserialize binary back into Store
    let store = rmp_serde::from_slice(&buffer)?;

    Ok(store)
}
