use dashmap::DashMap;
use std::collections::HashMap;

// =========================================================
// STORE STRUCT
// ---------------------------------------------------------
// DashMap is NOT serializable directly,
// so we keep it internal and convert when persisting.
// =========================================================
pub struct Store {
    map: DashMap<String, String>,
}

impl Store {
    // Create empty store
    pub fn new() -> Self {
        Self {
            map: DashMap::new(),
        }
    }

    // Insert or overwrite a key
    pub fn set(&self, key: String, value: String) {
        self.map.insert(key, value);

        // =============================================
        // OPTIONAL: Persist snapshot on every SET
        // =============================================
        if let Err(e) = crate::persistence::save(self, "db.bin") {
            eprintln!("Failed to persist DB: {}", e);
        }
    }

    // Retrieve value (cloned to avoid lifetime issues)
    pub fn get(&self, key: &str) -> Option<String> {
        self.map.get(key).map(|v| v.clone())
    }

    // Convert DashMap to HashMap (for persistence)
    pub fn to_hashmap(&self) -> HashMap<String, String> {
        self.map
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    // Load from HashMap to DashMap
    pub fn from_hashmap(data: HashMap<String, String>) -> Self {
        let map = DashMap::new();
        for (k, v) in data {
            map.insert(k, v);
        }
        Self { map }
    }
}
