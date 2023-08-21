use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use text_colorizer::*;

use crate::company::create_company_prompts;
use crate::db::run_migration;
use crate::utils::get_input;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub setup_done: bool,
    pub database_url: String,
    pub default_company: i32,
    pub logo_path: String,
    pub invoice_path: String,
}

pub fn is_setup_done() -> bool {
    let config_path = Path::new("config.json");
    let path_exists = config_path.exists();

    if !path_exists {
        return false;
    }

    let config_json = fs::read_to_string("config.json").expect("Unable to read file");
    let config: Config = serde_json::from_str(&config_json).expect("Unable to parse config.json");

    config.setup_done
}

pub fn load_config() -> Config {
    let config_json = fs::read_to_string("config.json").expect("Unable to read file");
    serde_json::from_str(&config_json).expect("Unable to parse config.json")
}

pub fn setup() -> Config {
    let default_path = dirs::home_dir()
        .expect("Home directory not found")
        .join("clinvoice")
        .display()
        .to_string();

    let mut db_url = get_input(&format!(
        "Database store path (default - {}): ",
        default_path.bright_black()
    ));

    if db_url.trim().is_empty() {
        db_url = default_path.clone();

        if !Path::new(&db_url).exists() {
            fs::create_dir_all(&db_url).expect("Failed to create directory");
        }
    }

    while !Path::new(&db_url).exists() {
        eprintln!("{}", "The database path does not exist.".red());
        db_url = get_input(&format!(
            "Database store path (default - {}): ",
            default_path.bright_black()
        ));
        if db_url.trim().is_empty() {
            db_url = default_path.clone();
        }
    }

    let db_file_path = Path::new(&db_url)
        .join("cli_invoice.sqlite3")
        .display()
        .to_string();

    // Create a new SQLite file if it doesn't exist
    if !Path::new(&db_file_path).exists() {
        let _ = fs::File::create(&db_file_path);
    }

    let mut invoice_save_path = get_input(&format!(
        "Invoice save path (default - {}): ",
        db_url.bright_black() // Using the db_url as the default path for invoices
    ));

    if invoice_save_path.trim().is_empty() {
        invoice_save_path = db_url;
    }

    // Ensure the invoice save directory exists
    if !Path::new(&invoice_save_path).exists() {
        fs::create_dir_all(&invoice_save_path).expect("Failed to create invoice directory");
    }

    // Run migrations
    let mut conn = SqliteConnection::establish(&db_file_path)
        .unwrap_or_else(|_| panic!("{} {}", "Error connecting to".red(), &db_file_path));

    run_migration(&mut conn);

    let config = Config {
        setup_done: false,
        database_url: db_file_path,
        default_company: -1,
        logo_path: String::new(),
        invoice_path: invoice_save_path,
    };
    let config_json = serde_json::to_string(&config).unwrap();
    fs::write("config.json", config_json).unwrap();

    let company = create_company_prompts(None, None, None, None);

    let mut logo_path = get_input("Path to your logo (png/jpeg/svg): ");

    while !is_valid_logo_path(&logo_path) {
        eprintln!(
            "{}",
            "Invalid logo path. Please provide a valid png, jpeg, or svg file.".red()
        );
        logo_path = get_input("Path to your logo (png/jpeg/svg): ");
    }

    let mut updated_config = load_config();
    updated_config.setup_done = true;
    updated_config.default_company = company.id;
    updated_config.logo_path = logo_path;
    let updated_config_json = serde_json::to_string(&updated_config).unwrap();
    fs::write("config.json", updated_config_json).unwrap();

    println!("{}", "Setup done".green());
    config
}

fn is_valid_logo_path(path: &str) -> bool {
    if !Path::new(path).exists() {
        return false;
    }

    matches!(
        Path::new(path).extension().and_then(|s| s.to_str()),
        Some("png") | Some("jpeg") | Some("jpg") | Some("svg")
    )
}
