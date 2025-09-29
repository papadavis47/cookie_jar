use libsql::Builder;
use std::time::Duration;

pub struct Database {
    db: libsql::Database,
}

impl Database {
    pub async fn new() -> Result<Self, libsql::Error> {
        let url = std::env::var("TURSO_DATABASE_URL").expect("TURSO_DATABASE_URL must be set");
        let token = std::env::var("TURSO_AUTH_TOKEN").expect("TURSO_AUTH_TOKEN must be set");

        let db = Builder::new_remote_replica("local.db", url, token)
            .sync_interval(Duration::from_secs(60)) // Auto-sync every 60 seconds
            .build()
            .await?;

        Ok(Self { db })
    }

    pub fn connect(&self) -> Result<libsql::Connection, libsql::Error> {
        self.db.connect()
    }

    pub async fn sync(&self) -> Result<(), libsql::Error> {
        self.db.sync().await
    }
}

// Example usage functions
pub async fn create_users_table(conn: &libsql::Connection) -> Result<(), libsql::Error> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL
        )",
        (),
    )
    .await?;
    Ok(())
}

pub async fn insert_user(conn: &libsql::Connection, name: &str) -> Result<(), libsql::Error> {
    conn.execute(
        "INSERT INTO users (name) VALUES (:name)",
        libsql::named_params! { ":name": name },
    )
    .await?;
    Ok(())
}

pub async fn get_all_users(conn: &libsql::Connection) -> Result<Vec<(i64, String)>, libsql::Error> {
    let mut rows = conn.query("SELECT id, name FROM users", ()).await?;
    let mut users = Vec::new();

    while let Some(row) = rows.next().await? {
        let id: i64 = row.get(0)?;
        let name: String = row.get(1)?;
        users.push((id, name));
    }

    Ok(users)
}
