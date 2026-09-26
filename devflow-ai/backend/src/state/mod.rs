use sqlx::SqlitePool;
use std::sync::Arc;

use crate::config::Config;

pub struct AppState {
    pub db: SqlitePool,
    pub config: Config,
}

impl AppState {
    pub fn new(db: SqlitePool, config: Config) -> Self {
        Self { db, config }
    }
}

pub type SharedState = Arc<AppState>;
