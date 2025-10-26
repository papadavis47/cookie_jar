use anyhow::{Context, Result};
use std::path::PathBuf;

/// Get the path to the cookie_jar directory ($HOME/.cookie_jar)
pub fn get_cookiejar_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .context("Could not determine home directory. Please set HOME or USERPROFILE environment variable.")?;

    let mut path = PathBuf::from(home);
    path.push(".cookie_jar");

    Ok(path)
}

/// Get the path to the local database file
pub fn get_db_path() -> Result<PathBuf> {
    let mut dir = get_cookiejar_dir()?;
    dir.push("cookie_jar.db");
    Ok(dir)
}

/// Get the path to the .env file
pub fn get_env_path() -> Result<PathBuf> {
    let mut dir = get_cookiejar_dir()?;
    dir.push(".env");
    Ok(dir)
}

/// Ensure the cookie_jar directory exists, create it if it doesn't
pub fn ensure_cookiejar_dir() -> Result<PathBuf> {
    let dir = get_cookiejar_dir()?;

    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .context(format!("Failed to create directory: {}", dir.display()))?;
    }

    Ok(dir)
}
