mod db;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Create database instance
    let database = db::Database::new().await?;

    // Get a connection
    let conn = database.connect()?;

    // Create the users table if it doesn't exist
    db::create_users_table(&conn).await?;

    // Insert a user
    db::insert_user(&conn, "Alice").await?;
    println!("✓ Inserted user");

    // Get all users
    let users = db::get_all_users(&conn).await?;
    println!("\nUsers in database:");
    for (id, name) in users {
        println!("  {} - {}", id, name);
    }

    // Manually sync with remote (auto-sync is already configured)
    database.sync().await?;
    println!("\n✓ Synced with remote database");

    Ok(())
}
