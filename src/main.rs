mod config;
mod db;
mod menu;
mod models;

use anyhow::Result;
use colored::*;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType},
};
use std::io::stdout;

#[tokio::main]
async fn main() -> Result<()> {
    // Ensure .cookiejar directory exists
    config::ensure_cookiejar_dir()?;

    // Load environment variables from .env file in ~/.cookie_jar/
    let env_path = config::get_env_path()?;
    dotenvy::from_path(&env_path).ok();

    // Get database path
    let db_path = config::get_db_path()?;

    // Create database instance with local replica
    let database = db::Database::new(db_path).await?;

    // Get a connection
    let conn = database.connect()?;

    // Initialize schema (creates tables if they don't exist)
    db::init_schema(&conn).await?;

    // Initial sync with Turso Cloud
    database.sync().await?;

    // Enter alternate screen buffer (like vim)
    execute!(stdout(), EnterAlternateScreen, Clear(ClearType::All))?;

    // Main menu loop
    let result = async {
        loop {
            match menu::show_main_menu(&conn, &database).await {
                Ok(should_exit) => {
                    if should_exit {
                        // Sync one final time before exiting
                        database.sync().await?;
                        println!("\n{} Goodbye!", "👋".bright_white());
                        break;
                    }
                    // After each operation, sync with remote
                    database.sync().await?;
                }
                Err(e) => {
                    eprintln!("\n{} Error: {:?}", "✗".bright_red(), e);
                    // Continue running even if there's an error
                }
            }
        }
        Ok::<(), anyhow::Error>(())
    }.await;

    // Always exit alternate screen buffer, even on error
    execute!(stdout(), LeaveAlternateScreen)?;

    result
}
