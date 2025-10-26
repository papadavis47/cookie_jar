use crate::models::{Bucket, Cookie};
use anyhow::{Context, Result};
use libsql::Builder;
use std::path::PathBuf;
use std::time::Duration;

pub struct Database {
    db: libsql::Database,
}

impl Database {
    /// Creates a new Database instance with local replica and Turso sync
    /// The local database will be stored in $HOME/.cookiejar/cookiejar.db
    pub async fn new(local_path: PathBuf) -> Result<Self> {
        let url = std::env::var("TURSO_DATABASE_URL")
            .context("TURSO_DATABASE_URL must be set in .env file")?;
        let token = std::env::var("TURSO_AUTH_TOKEN")
            .context("TURSO_AUTH_TOKEN must be set in .env file")?;

        let db = Builder::new_remote_replica(local_path, url, token)
            .sync_interval(Duration::from_secs(60)) // Auto-sync every 60 seconds
            .build()
            .await
            .context("Failed to create database")?;

        Ok(Self { db })
    }

    pub fn connect(&self) -> Result<libsql::Connection> {
        self.db.connect().context("Failed to connect to database")
    }

    pub async fn sync(&self) -> Result<()> {
        self.db.sync().await.context("Failed to sync with remote")?;
        Ok(())
    }
}

/// Initialize the database schema (buckets and cookies tables)
pub async fn init_schema(conn: &libsql::Connection) -> Result<()> {
    // Create buckets table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS buckets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE NOT NULL,
            created_at INTEGER NOT NULL
        )",
        (),
    )
    .await
    .context("Failed to create buckets table")?;

    // Create cookies table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS cookies (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            bucket_id INTEGER NOT NULL,
            content TEXT NOT NULL CHECK(length(content) <= 300),
            created_at INTEGER NOT NULL,
            FOREIGN KEY (bucket_id) REFERENCES buckets(id)
        )",
        (),
    )
    .await
    .context("Failed to create cookies table")?;

    Ok(())
}

// ============ BUCKET OPERATIONS ============

/// Create a new bucket
pub async fn create_bucket(conn: &libsql::Connection, name: &str) -> Result<Bucket> {
    let timestamp = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT INTO buckets (name, created_at) VALUES (?1, ?2)",
        libsql::params![name, timestamp],
    )
    .await
    .context("Failed to create bucket")?;

    // Get the last inserted row ID
    let mut rows = conn.query("SELECT last_insert_rowid()", ()).await?;
    if let Some(row) = rows.next().await? {
        let id: i64 = row.get(0)?;
        Ok(Bucket::new(id, name.to_string(), timestamp))
    } else {
        anyhow::bail!("Failed to get bucket ID after insert")
    }
}

/// Get all buckets
pub async fn get_all_buckets(conn: &libsql::Connection) -> Result<Vec<Bucket>> {
    let mut rows = conn
        .query("SELECT id, name, created_at FROM buckets ORDER BY name", ())
        .await
        .context("Failed to query buckets")?;

    let mut buckets = Vec::new();
    while let Some(row) = rows.next().await? {
        let id: i64 = row.get(0)?;
        let name: String = row.get(1)?;
        let created_at: i64 = row.get(2)?;
        buckets.push(Bucket::new(id, name, created_at));
    }

    Ok(buckets)
}

/// Count cookies in a bucket
pub async fn count_cookies_in_bucket(conn: &libsql::Connection, bucket_id: i64) -> Result<i64> {
    let mut rows = conn
        .query(
            "SELECT COUNT(*) FROM cookies WHERE bucket_id = ?1",
            libsql::params![bucket_id],
        )
        .await?;

    if let Some(row) = rows.next().await? {
        let count: i64 = row.get(0)?;
        Ok(count)
    } else {
        Ok(0)
    }
}

// ============ COOKIE OPERATIONS ============

/// Create a new cookie
pub async fn create_cookie(conn: &libsql::Connection, bucket_id: i64, content: &str) -> Result<i64> {
    if content.len() > 300 {
        anyhow::bail!("Cookie content must be 300 characters or less");
    }

    let timestamp = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT INTO cookies (bucket_id, content, created_at) VALUES (?1, ?2, ?3)",
        libsql::params![bucket_id, content, timestamp],
    )
    .await
    .context("Failed to create cookie")?;

    // Get the last inserted row ID
    let mut rows = conn.query("SELECT last_insert_rowid()", ()).await?;
    if let Some(row) = rows.next().await? {
        let id: i64 = row.get(0)?;
        Ok(id)
    } else {
        anyhow::bail!("Failed to get cookie ID after insert")
    }
}

/// Get all cookies
pub async fn get_all_cookies(conn: &libsql::Connection) -> Result<Vec<Cookie>> {
    let mut rows = conn
        .query(
            "SELECT id, bucket_id, content, created_at FROM cookies ORDER BY created_at DESC",
            (),
        )
        .await
        .context("Failed to query cookies")?;

    let mut cookies = Vec::new();
    while let Some(row) = rows.next().await? {
        let id: i64 = row.get(0)?;
        let bucket_id: i64 = row.get(1)?;
        let content: String = row.get(2)?;
        let created_at: i64 = row.get(3)?;
        cookies.push(Cookie::new(id, bucket_id, content, created_at));
    }

    Ok(cookies)
}

/// Get cookies by bucket ID
pub async fn get_cookies_by_bucket(conn: &libsql::Connection, bucket_id: i64) -> Result<Vec<Cookie>> {
    let mut rows = conn
        .query(
            "SELECT id, bucket_id, content, created_at FROM cookies WHERE bucket_id = ?1 ORDER BY created_at DESC",
            libsql::params![bucket_id],
        )
        .await
        .context("Failed to query cookies by bucket")?;

    let mut cookies = Vec::new();
    while let Some(row) = rows.next().await? {
        let id: i64 = row.get(0)?;
        let bucket_id: i64 = row.get(1)?;
        let content: String = row.get(2)?;
        let created_at: i64 = row.get(3)?;
        cookies.push(Cookie::new(id, bucket_id, content, created_at));
    }

    Ok(cookies)
}
