use crate::resp;
use crate::store::Store;

// =========================================================
// SUPPORTED COMMANDS
// =========================================================
pub enum Command {
    Set(String, String, Option<u64>),
    Get(String),
}

impl Command {
    // Parse vec input into structured command
    pub fn from_vec(args: Vec<String>) -> Result<Self, &'static str> {
        if args.is_empty() {
            return Err("ERR empty command");
        }

        let cmd = args[0].to_uppercase();

        match cmd.as_str() {
            "GET" if args.len() == 2 => Ok(Command::Get(args[1].clone())),

            "SET" if args.len() == 3 => Ok(Command::Set(args[1].clone(), args[2].clone(), None)),

            "SET" if args.len() == 5 && args[3].to_uppercase() == "EX" => {
                let ttl = args[4].parse::<u64>().map_err(|_| "ERR invalid TTL")?;

                Ok(Command::Set(args[1].clone(), args[2].clone(), Some(ttl)))
            }

            _ => Err("ERR unknown command"),
        }
    }

    // Execute command against the Store
    pub fn execute(self, store: &Store) -> String {
        match self {
            Command::Set(key, value, ttl) => {
                store.set(key, value, ttl);
                resp::encode_simple_string("OK")
            }
            Command::Get(key) => match store.get(&key) {
                Some(val) => resp::encode_bulk_string(&val),
                None => resp::encode_null(),
            },
        }
    }
}
