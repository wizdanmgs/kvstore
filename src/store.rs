use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =========================================================
// IN-MEMORY KEY-VALUE STORE
// =========================================================
// Deriving Serialize/Deserialize allows snapshot persistence
// =========================================================
#[derive(Serialize, Deserialize)]
pub struct Store {
    map: HashMap<String, String>,
}

impl Store {
    // Create empty store
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    // Insert or overwrite a key
    pub fn set(&mut self, key: String, value: String) {
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
        self.map.get(key).cloned()
    }
}
