use crate::store::Store;

// =========================================================
// SUPPORTED COMMANDS
// =========================================================
pub enum Command {
    Set(String, String),
    Get(String),
}

impl Command {
    // Parse raw string input into structured command
    pub fn parse(input: &str) -> Result<Self, &'static str> {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();

        match parts.as_slice() {
            ["SET", key, value] => Ok(Command::Set(key.to_string(), value.to_string())),
            ["GET", key] => Ok(Command::Get(key.to_string())),
            _ => Err("Invalid command"),
        }
    }

    // Execute command against the Store
    pub fn execute(self, store: &Store) -> String {
        match self {
            Command::Set(key, value) => {
                store.set(key, value);
                "OK\n".into()
            }
            Command::Get(key) => match store.get(&key) {
                Some(val) => format!("{}\n", val),
                None => "nil\n".into(),
            },
        }
    }
}
