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
    pub last_accessed: u64,
}

pub struct Store {
    map: DashMap<String, Entry>,
    max_keys: usize,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

impl Store {
    // =====================================================
    // Create empty store
    // =====================================================
    pub fn new(max_keys: usize) -> Self {
        Self {
            map: DashMap::new(),
            max_keys,
        }
    }
    // =====================================================
    // Insert or overwrite a key with optional TTL
    // =====================================================
    pub fn set(&self, key: String, value: String, ttl: Option<u64>) {
        let expires_at = ttl.map(|seconds| now() + seconds);

        let entry = Entry {
            value,
            expires_at,
            last_accessed: now(),
        };

        // Append to WAL (Write-Ahead Log) before modifying memory
        if let Err(e) = crate::wal::append_set(&key, &entry.value, ttl) {
            eprintln!("Failed to write WAL: {}", e);
            return;
        }

        self.map.insert(key, entry);

        // Check LRU eviction
        self.evict_if_needed();

        // Persist snapshot on every SET
        if let Err(e) = crate::persistence::save(self, "db.bin") {
            eprintln!("Failed to persist DB: {}", e);
        }
    }

    // =====================================================
    // Internal set for WAL replay (no WAL write)
    // =====================================================
    pub fn set_internal(&self, key: String, value: String, ttl: Option<u64>) {
        let expires_at = ttl.map(|seconds| now() + seconds);
        let entry = Entry {
            value,
            expires_at,
            last_accessed: now(),
        };
        self.map.insert(key, entry);
    }

    // =====================================================
    // Retrieve value with lazy expiration (cloned to avoid lifetime issues)
    // =====================================================
    pub fn get(&self, key: &str) -> Option<String> {
        if let Some(mut entry) = self.map.get_mut(key) {
            // Check expiration
            if let Some(exp) = entry.expires_at {
                if now() > exp {
                    // Remove expired key
                    self.map.remove(key);
                    return None;
                }
            }

            // Update last accessed
            entry.last_accessed = now();

            return Some(entry.value.clone());
        }
        None
    }

    // =====================================================
    // Background expiration cleanup
    // =====================================================
    pub fn cleanup_expired(&self) {
        let now = now();

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
    // Evict least used key from cache
    // =====================================================
    pub fn evict_if_needed(&self) {
        if self.map.len() <= self.max_keys {
            return;
        }

        // Find least recently used key
        if let Some(lru_key) = self
            .map
            .iter()
            .min_by_key(|entry| entry.last_accessed)
            .map(|entry| entry.key().clone())
        {
            println!("Evicting key: {}", lru_key);
            self.map.remove(&lru_key);
        }
    }

    // =====================================================
    // Convert DashMap to HashMap (for persistence)
    // =====================================================
    pub fn to_hashmap(&self) -> HashMap<String, (String, Option<u64>, u64)> {
        self.map
            .iter()
            .map(|entry| {
                (
                    entry.key().clone(),
                    (
                        entry.value().value.clone(),
                        entry.value().expires_at,
                        entry.value().last_accessed,
                    ),
                )
            })
            .collect()
    }

    // =====================================================
    // Load from HashMap to DashMap
    // =====================================================
    pub fn from_hashmap(data: HashMap<String, (String, Option<u64>, u64)>) -> Self {
        let map = DashMap::new();
        for (key, (value, expires_at, last_accessed)) in data {
            map.insert(
                key,
                Entry {
                    value,
                    expires_at,
                    last_accessed,
                },
            );
        }

        Self {
            map,
            max_keys: 10_000,
        }
    }
}
