# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**cookie_jar** (`cj`) is a CLI tool for storing and organizing "cookies" (achievements, proud moments, or text snippets up to 300 characters) into "buckets" (categories). It uses a cloud-synced local SQLite database via Turso.

## Development Commands

```bash
# Run the application in development
cargo run

# Build the binary (debug mode)
cargo build

# Build optimized release binary
cargo build --release

# Install binary to ~/.cargo/bin (use --force to overwrite existing)
cargo install --path . --force

# Run the installed binary
cj
```

## Architecture

### Core Components

- **main.rs** - Application entry point. Initializes database, establishes connection, syncs with Turso, and runs the main menu loop.
- **db.rs** - Database layer with all CRUD operations for buckets and cookies. Handles libsql connection and sync.
- **menu.rs** - TUI layer using dialoguer. Implements vim-style navigation (j/k keys) and all interactive flows.
- **models.rs** - Domain models: `Bucket` and `Cookie` with timestamp conversion logic.
- **config.rs** - Path management for `~/.cookie_jar/` directory, database file, and .env file.

### Database Architecture

Uses **libsql** (Turso) with a local embedded replica pattern:
- Local SQLite file: `~/.cookie_jar/cookie_jar.db`
- Environment variables loaded from: `~/.cookie_jar/.env`
- Syncs with Turso cloud every 60 seconds automatically
- **Critical:** Manual sync immediately after bucket creation to ensure foreign key constraints work
- Additional sync after each menu operation in main loop
- Requires `TURSO_DATABASE_URL` and `TURSO_AUTH_TOKEN` in `.env`

**Schema:**
```sql
CREATE TABLE buckets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    created_at INTEGER NOT NULL  -- Unix timestamp
);

CREATE TABLE cookies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bucket_id INTEGER NOT NULL,
    content TEXT NOT NULL CHECK(length(content) <= 300),
    created_at INTEGER NOT NULL,  -- Unix timestamp
    FOREIGN KEY (bucket_id) REFERENCES buckets(id)
);
```

### Key Design Patterns

1. **Async throughout** - Uses Tokio runtime, all DB operations are async
2. **Error propagation** - Uses `anyhow::Result` for error handling with context
3. **No ORM** - Raw SQL queries via libsql
4. **Timestamps** - Stored as i64 Unix timestamps, converted to `DateTime<Utc>` in models

### Important Implementation Details

- **Replication timing is critical:**
  - `create_bucket` returns a full `Bucket` object (not just ID) to avoid read-after-write issues
  - `select_or_create_bucket` returns `Bucket` objects (not IDs) for the same reason
  - Database must sync immediately after bucket creation before creating cookies, otherwise foreign key constraints fail
  - The `Database` struct is passed to menu functions to enable sync operations
- **Removed functions:** `get_bucket_by_id` was removed as it's no longer needed (replaced by returning full objects)
- Vim-style navigation (j/k) implemented via custom `VimTheme` for dialoguer
- Pastel colors assigned to buckets based on bucket ID modulo color array length
- Cookie content validated at both application (dialoguer) and database (CHECK constraint) levels
- `Cookie.id` field has `#[allow(dead_code)]` as it's retained for future edit/delete features but not currently used

## Environment Setup

The application requires a `.env` file located at `~/.cookie_jar/.env` with the following contents:
```
TURSO_DATABASE_URL=libsql://your-database.turso.io
TURSO_AUTH_TOKEN=your-token-here
```

**Important:** The `.env` file must be placed in `~/.cookie_jar/.env` (not in the project directory). This ensures the installed binary can find the credentials regardless of where it's executed from.

The application will fail to start without these credentials.

## Binary Name

The compiled binary is named `cj` (not `cookie_jar`), configured in `Cargo.toml`.
- This project will change frequently. After changes are made to the codebase, Claude should update the Claude.md file to reflect the changes - to ensure clarity for agents going forward.