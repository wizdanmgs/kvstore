# Kvstore

Kvstore is a simplified Redis-like in-memory key-value store written in Rust.  
It supports persistence, write-ahead logging (WAL), TTL expiration, LRU eviction, and the Redis RESP2 protocol.

---

## Features

- In-memory key-value storage
- Redis RESP2 protocol support (compatible with `redis-cli`)
- SET / GET commands
- Optional TTL with expiration worker
- LRU eviction policy
- Write-Ahead Log (WAL)
- Snapshot persistence to disk
- Graceful shutdown handling
- Concurrent access using DashMap
- Configurable bind address via CLI

---

## Architecture Overview

Client (redis-cli)
│
▼
TCP Server (Tokio)
│
▼
RESP Parser
│
▼
Command Executor
│
▼
In-Memory Store (DashMap)
│
├── WAL (append-only log)
└── Snapshot (binary persistence)

---

## Installation

### Requirements

- Rust (stable)
- Cargo

### Clone and Build

```sh
git clone https://github.com/wizdanmgs/kvstore
cd kvstore
cargo build --release
```

---

## Running the Server

### Default configuration:

```sh
cargo run
```

### Custom address:

```sh
cargo run -- --addr 0.0.0.0:7000
```

### Custom database file:

```sh
cargo run -- --db data.bin
```

### Both:

```sh
cargo run -- --addr 0.0.0.0:7000 --db prod.bin
```

### Default values:

- Address: `127.0.0.1:6380`
- Database file: `db.bin`

---

## Using redis-cli

Start the server, then connect:

```sh
redis-cli -p 6380
```

Example commands:

```sh
SET name wizdan
GET name
```

```sh
SET temp value EX 10
GET temp
```

TTL expiration is enforced both lazily (on GET) and actively (background worker).

---

## Supported Commands

### SET

```sh
SET key value
SET key value EX seconds
```

### GET

```sh
GET key
```

If the key does not exist, a RESP null bulk string is returned.

---

## Persistence

Kvstore uses two mechanisms:

### 1. Write-Ahead Log (WAL)

Every mutation is appended to a log file before being applied to memory.  
This ensures durability in case of crashes.

### 2. Snapshot

On shutdown (or defined event), the in-memory state is serialized to disk in a binary format.

On startup:

1. Snapshot is loaded (if present)
2. WAL is replayed
3. Server begins accepting connections

---

## TTL and Expiration

- TTL is stored as an absolute UNIX timestamp.
- Expired keys are:
  - Removed lazily on access
  - Cleaned periodically by a background worker

This ensures expiration survives server restarts.

---

## LRU Eviction

When a maximum capacity is configured:

- Least Recently Used keys are evicted
- Access updates usage order
- Eviction happens on insert when capacity is exceeded

This prevents unbounded memory growth.

---

## CLI Options

```yaml
USAGE:
kvstore [OPTIONS]

OPTIONS:
-a, --addr <ADDR> Address to bind (default: 127.0.0.1:6379)
-d, --db <FILE> Database file path (default: db.bin)
-h, --help Print help
-V, --version Print version
```
