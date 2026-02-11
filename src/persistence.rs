use crate::store::Store;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};

// =========================================================
// SAVE STORE TO DISK (SNAPSHOT)
// =========================================================
pub fn save(store: &Store, path: &str) -> anyhow::Result<()> {
    // Convert DashMap to HashMap
    let snapshot: HashMap<String, String> = store.to_hashmap();

    // Serialize store into compact binary (MessagePack)
    let encoded = rmp_serde::to_vec(&snapshot)?;

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
    let store: HashMap<String, String> = rmp_serde::from_slice(&buffer)?;

    Ok(Store::from_hashmap(store))
}
