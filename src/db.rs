use crate::config;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use text_colorizer::*;

pub fn establish_connection() -> SqliteConnection {
    let config = config::load_config();

    SqliteConnection::establish(&config.database_url)
        .unwrap_or_else(|_| panic!("{} {}", "Error connecting to".red(), config.database_url))
}

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub fn run_migration(conn: &mut SqliteConnection) {
    conn.run_pending_migrations(MIGRATIONS).unwrap();
}
