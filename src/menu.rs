use crate::db;
use crate::models::Bucket;
use anyhow::Result;
use colored::*;
use crossterm::{execute, terminal::{Clear, ClearType}, cursor::MoveTo};
use dialoguer::{theme::ColorfulTheme, Input, Select};
use std::io::{stdout, stdin, Write};

/// Main menu options
#[derive(Debug)]
enum MainMenuOption {
    AddCookie,
    ViewAllCookies,
    ViewCookiesByBucket,
    ListBuckets,
    Exit,
}

impl std::fmt::Display for MainMenuOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MainMenuOption::AddCookie => write!(f, "Add a new cookie"),
            MainMenuOption::ViewAllCookies => write!(f, "View all cookies"),
            MainMenuOption::ViewCookiesByBucket => write!(f, "View cookies by bucket"),
            MainMenuOption::ListBuckets => write!(f, "List all buckets"),
            MainMenuOption::Exit => write!(f, "Exit"),
        }
    }
}

/// Pastel colors for buckets (cycling through these)
const PASTEL_COLORS: &[&str] = &[
    "bright cyan",
    "bright magenta",
    "bright yellow",
    "bright green",
    "bright blue",
];

/// Get a consistent pastel color for a bucket based on its ID
fn get_bucket_color(bucket_id: i64) -> Color {
    let index = (bucket_id as usize) % PASTEL_COLORS.len();
    match PASTEL_COLORS[index] {
        "bright cyan" => Color::BrightCyan,
        "bright magenta" => Color::BrightMagenta,
        "bright yellow" => Color::BrightYellow,
        "bright green" => Color::BrightGreen,
        "bright blue" => Color::BrightBlue,
        _ => Color::White,
    }
}

/// Wait for user to press Enter before continuing
fn wait_for_enter() -> Result<()> {
    print!("\n{}", "Press Enter to continue...".bright_white());
    stdout().flush()?;
    let mut buffer = String::new();
    stdin().read_line(&mut buffer)?;
    Ok(())
}

/// Custom theme with vim keybindings
struct VimTheme;

impl dialoguer::theme::Theme for VimTheme {
    fn format_prompt(&self, f: &mut dyn std::fmt::Write, prompt: &str) -> std::fmt::Result {
        write!(f, "{}", prompt.bright_white().bold())
    }

    fn format_select_prompt_item(
        &self,
        f: &mut dyn std::fmt::Write,
        text: &str,
        active: bool,
    ) -> std::fmt::Result {
        if active {
            write!(f, "{} {}", "→".bright_cyan().bold(), text)
        } else {
            write!(f, "  {}", text)
        }
    }
}

/// Display the main menu and handle user selection
pub async fn show_main_menu(conn: &libsql::Connection, db: &crate::db::Database) -> Result<bool> {
    // Clear screen and move cursor to top before showing menu
    execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0))?;

    println!("{}", "╔═══════════════════╗".bright_white().bold());
    println!("{}", "║   C O O K I E     ║".bright_white().bold());
    println!("{}", "║      J A R        ║".bright_white().bold());
    println!("{}", "╚═══════════════════╝".bright_white().bold());
    println!();
    println!("{}", "What would you like to do?".bright_white());
    println!();
    println!("{}", "(use j/k or arrow keys to navigate)".bright_black());
    println!();

    let options = vec![
        MainMenuOption::AddCookie,
        MainMenuOption::ViewAllCookies,
        MainMenuOption::ViewCookiesByBucket,
        MainMenuOption::ListBuckets,
        MainMenuOption::Exit,
    ];

    let selection = Select::with_theme(&VimTheme)
        .items(&options)
        .default(0)
        .interact()?;

    match options[selection] {
        MainMenuOption::AddCookie => add_cookie_flow(conn, db).await?,
        MainMenuOption::ViewAllCookies => view_all_cookies(conn).await?,
        MainMenuOption::ViewCookiesByBucket => view_cookies_by_bucket_flow(conn).await?,
        MainMenuOption::ListBuckets => list_buckets(conn).await?,
        MainMenuOption::Exit => return Ok(true), // Signal to exit
    }

    Ok(false) // Continue running
}


/// Flow for adding a new cookie
async fn add_cookie_flow(conn: &libsql::Connection, db: &crate::db::Database) -> Result<()> {
    // Get all existing buckets
    let buckets = db::get_all_buckets(conn).await?;

    let bucket = if buckets.is_empty() {
        // No buckets exist, create first one
        println!("\n{}", "No buckets exist yet. Let's create your first bucket!".bright_yellow());
        let bucket_name: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Bucket name")
            .interact_text()?;

        let bucket = db::create_bucket(conn, &bucket_name).await?;
        // Sync immediately after bucket creation to ensure foreign key constraints work
        db.sync().await?;
        bucket
    } else {
        // Show existing buckets + option to create new
        select_or_create_bucket(conn, db, &buckets).await?
    };

    // Get cookie content
    let content: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Enter your cookie (max 300 chars)"))
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.is_empty() {
                Err("Cookie cannot be empty")
            } else if input.len() > 300 {
                Err("Cookie must be 300 characters or less")
            } else {
                Ok(())
            }
        })
        .interact_text()?;

    // Create the cookie
    db::create_cookie(conn, bucket.id, &content).await?;

    println!(
        "\n{} Cookie added to \"{}\" bucket!",
        "✨".bright_green(),
        bucket.name.color(get_bucket_color(bucket.id)).bold()
    );

    Ok(())
}

/// Select an existing bucket or create a new one
async fn select_or_create_bucket(conn: &libsql::Connection, db: &crate::db::Database, buckets: &[Bucket]) -> Result<Bucket> {
    println!("\n{}", "Available buckets:".bright_white());
    println!("{}", "(use j/k or arrow keys to navigate)".bright_black());

    let mut items: Vec<String> = Vec::new();

    for bucket in buckets {
        let count = db::count_cookies_in_bucket(conn, bucket.id).await?;
        let colored_name = bucket.name.color(get_bucket_color(bucket.id)).bold().to_string();
        items.push(format!("{} ({} cookies)", colored_name, count));
    }

    items.push("+ Create new bucket".bright_green().to_string());

    let selection = Select::with_theme(&VimTheme)
        .items(&items)
        .default(0)
        .interact()?;

    if selection == items.len() - 1 {
        // Create new bucket
        let bucket_name: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("New bucket name")
            .interact_text()?;

        let bucket = db::create_bucket(conn, &bucket_name).await?;
        println!(
            "{} Created bucket \"{}\"",
            "✓".bright_green(),
            bucket.name.color(get_bucket_color(bucket.id)).bold()
        );
        // Sync immediately after bucket creation to ensure foreign key constraints work
        db.sync().await?;
        Ok(bucket)
    } else {
        // Use existing bucket
        Ok(buckets[selection].clone())
    }
}

/// View all cookies
async fn view_all_cookies(conn: &libsql::Connection) -> Result<()> {
    let cookies = db::get_all_cookies(conn).await?;
    let buckets = db::get_all_buckets(conn).await?;

    if cookies.is_empty() {
        println!("\n{}", "No cookies yet! Add your first one.".bright_yellow());
        wait_for_enter()?;
        return Ok(());
    }

    println!("\n{}", "All Cookies:".bright_white().bold());
    println!("{}", "─".repeat(60).bright_black());

    for cookie in &cookies {
        // Find the bucket for this cookie
        let bucket = buckets.iter().find(|b| b.id == cookie.bucket_id);
        let bucket_name = bucket.map(|b| b.name.as_str()).unwrap_or("Unknown");
        let bucket_color = get_bucket_color(cookie.bucket_id);

        println!(
            "\n{} {}",
            "📌".bright_white(),
            bucket_name.color(bucket_color).bold()
        );
        println!("   {}", cookie.content.bright_white());
        println!(
            "   {} {}",
            "🕒".bright_black(),
            cookie.formatted_created_at().bright_black()
        );
    }

    println!("\n{}", "─".repeat(60).bright_black());
    println!("Total: {} cookies", cookies.len().to_string().bright_cyan().bold());

    wait_for_enter()?;

    Ok(())
}

/// Flow for viewing cookies by bucket
async fn view_cookies_by_bucket_flow(conn: &libsql::Connection) -> Result<()> {
    let buckets = db::get_all_buckets(conn).await?;

    if buckets.is_empty() {
        println!("\n{}", "No buckets exist yet!".bright_yellow());
        wait_for_enter()?;
        return Ok(());
    }

    println!("\n{}", "Select a bucket:".bright_white());
    println!("{}", "(use j/k or arrow keys to navigate)".bright_black());

    let items: Vec<String> = buckets
        .iter()
        .map(|b| {
            let colored_name = b.name.color(get_bucket_color(b.id)).bold().to_string();
            colored_name
        })
        .collect();

    let selection = Select::with_theme(&VimTheme)
        .items(&items)
        .default(0)
        .interact()?;

    let bucket = &buckets[selection];
    let cookies = db::get_cookies_by_bucket(conn, bucket.id).await?;

    if cookies.is_empty() {
        println!(
            "\n{} No cookies in \"{}\" yet!",
            "ℹ".bright_yellow(),
            bucket.name.color(get_bucket_color(bucket.id)).bold()
        );
        wait_for_enter()?;
        return Ok(());
    }

    println!(
        "\n{} {}",
        "Cookies in".bright_white(),
        bucket.name.color(get_bucket_color(bucket.id)).bold()
    );
    println!("{}", "─".repeat(60).bright_black());

    for cookie in &cookies {
        println!("\n   {}", cookie.content.bright_white());
        println!(
            "   {} {}",
            "🕒".bright_black(),
            cookie.formatted_created_at().bright_black()
        );
    }

    println!("\n{}", "─".repeat(60).bright_black());
    println!("Total: {} cookies", cookies.len().to_string().bright_cyan().bold());

    wait_for_enter()?;

    Ok(())
}

/// List all buckets with cookie counts
async fn list_buckets(conn: &libsql::Connection) -> Result<()> {
    let buckets = db::get_all_buckets(conn).await?;

    if buckets.is_empty() {
        println!("\n{}", "No buckets exist yet!".bright_yellow());
        wait_for_enter()?;
        return Ok(());
    }

    println!("\n{}", "All Buckets:".bright_white().bold());
    println!("{}", "─".repeat(60).bright_black());

    for bucket in &buckets {
        let count = db::count_cookies_in_bucket(conn, bucket.id).await?;
        println!(
            "\n{} {} - {} cookies",
            "📁".bright_white(),
            bucket.name.color(get_bucket_color(bucket.id)).bold(),
            count.to_string().bright_cyan()
        );
        println!(
            "   {} Created {}",
            "🕒".bright_black(),
            bucket.formatted_created_at().bright_black()
        );
    }

    println!("\n{}", "─".repeat(60).bright_black());
    println!("Total: {} buckets", buckets.len().to_string().bright_cyan().bold());

    wait_for_enter()?;

    Ok(())
}
