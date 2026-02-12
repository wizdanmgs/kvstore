use dashmap::DashMap;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// =========================================================
// STORE STRUCT
// ---------------------------------------------------------
// DashMap is NOT serializable directly,
// so we keep it internal and convert when persisting.
// =========================================================
pub struct Entry {
    pub value: String,
    pub expires_at: Option<u64>, // UNIX timestamp (seconds)
}

pub struct Store {
    map: DashMap<String, Entry>,
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

impl Store {
    // Create empty store
    pub fn new() -> Self {
        Self {
            map: DashMap::new(),
        }
    }
    // =====================================================
    // Insert or overwrite a key with optional TTL
    // =====================================================
    pub fn set(&self, key: String, value: String, ttl: Option<u64>) {
        let expires_at = ttl.map(|seconds| current_timestamp() + seconds);

        // Append to WAL (Write-Ahead Log) before modifying memory
        if let Err(e) = crate::wal::append_set(&key, &value, ttl) {
            eprintln!("Failed to write WAL: {}", e);
            return;
        }

        self.map.insert(key, Entry { value, expires_at });

        // Persist snapshot on every SET
        if let Err(e) = crate::persistence::save(self, "db.bin") {
            eprintln!("Failed to persist DB: {}", e);
        }
    }

    // Internal set for WAL replay (no WAL write)
    pub fn set_internal(&self, key: String, value: String, ttl: Option<u64>) {
        let expires_at = ttl.map(|seconds| current_timestamp() + seconds);
        self.map.insert(key, Entry { value, expires_at });
    }

    // =====================================================
    // Retrieve value with lazy expiration (cloned to avoid lifetime issues)
    // =====================================================
    pub fn get(&self, key: &str) -> Option<String> {
        if let Some(entry) = self.map.get(key) {
            // Check expiration
            if let Some(exp) = entry.expires_at {
                if current_timestamp() > exp {
                    // Remove expired key
                    self.map.remove(key);
                    return None;
                }
            }
            return Some(entry.value.clone());
        }
        None
    }

    // =====================================================
    // Background expiration cleanup
    // =====================================================
    pub fn cleanup_expired(&self) {
        let now = current_timestamp();

        let keys_to_remove: Vec<String> = self
            .map
            .iter()
            .filter_map(|entry| {
                if let Some(exp) = entry.expires_at {
                    if now > exp {
                        // Remove expired key
                        return Some(entry.key().clone());
                    }
                }
                None
            })
            .collect();

        for key in keys_to_remove {
            self.map.remove(&key);
        }
    }

    // =====================================================
    // Convert DashMap to HashMap (for persistence)
    // =====================================================
    pub fn to_hashmap(&self) -> HashMap<String, (String, Option<u64>)> {
        self.map
            .iter()
            .map(|entry| {
                (
                    entry.key().clone(),
                    (entry.value().value.clone(), entry.value().expires_at),
                )
            })
            .collect()
    }

    // =====================================================
    // Load from HashMap to DashMap
    // =====================================================
    pub fn from_hashmap(data: HashMap<String, (String, Option<u64>)>) -> Self {
        let map = DashMap::new();
        for (key, (value, expires_at)) in data {
            map.insert(key, Entry { value, expires_at });
        }
        Self { map }
    }
}
