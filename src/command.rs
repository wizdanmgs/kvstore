use crate::store::Store;

// =========================================================
// SUPPORTED COMMANDS
// =========================================================
pub enum Command {
    Set(String, String, Option<u64>),
    Get(String),
}

impl Command {
    // Parse raw string input into structured command
    pub fn parse(input: &str) -> Result<Self, &'static str> {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();

        match parts.as_slice() {
            ["SET", key, value] => Ok(Command::Set(key.to_string(), value.to_string(), None)),
            ["SET", key, value, "EX", ttl] => {
                let ttl = ttl.parse::<u64>().map_err(|_| "Invalid TTL")?;
                Ok(Command::Set(key.to_string(), value.to_string(), Some(ttl)))
            }
            ["GET", key] => Ok(Command::Get(key.to_string())),
            _ => Err("Invalid command"),
        }
    }

    // Execute command against the Store
    pub fn execute(self, store: &Store) -> String {
        match self {
            Command::Set(key, value, ttl) => {
                store.set(key, value, ttl);
                "OK\n".into()
            }
            Command::Get(key) => match store.get(&key) {
                Some(val) => format!("{}\n", val),
                None => "nil\n".into(),
            },
        }
    }
}
