use actix_web::web;
use async_trait::async_trait;
pub mod mongodb;
pub mod sqlite;
use serde_json::Value;

use ::sqlite::Connection;
use std::error::Error;
use std::sync::Mutex;

const BRANCH_API: &str =
    "https://api.github.com/repos/davidmegginson/ourairports-data/branches/main";
const TREE_API: &str = "https://api.github.com/repos/davidmegginson/ourairports-data/git/trees/";

const CSV_ROOT_URL: &str =
    "https://raw.githubusercontent.com/davidmegginson/ourairports-data/main/";
const AIRPORT_CSV: &str = "airports.csv";
const AIRPORT_FREQUENCY_CSV: &str = "airport-frequencies.csv";
const AIRPORT_RUNWAY_CSV: &str = "runways.csv";
const NAVAID_CSV: &str = "navaids.csv";

pub struct AppState {
    pub sqlite_connection: Mutex<Connection>,
    pub database: Box<dyn DatabaseBackend + Sync + Send>,
}

pub async fn periodical_update(app_state: web::Data<AppState>) {
    app_state.clone().database.periodical_update().await
}

#[async_trait]
pub trait DatabaseBackend {
    async fn periodical_update(self: &Self) {}
    async fn get_airport_by_icao_code(self: &Self, icao: String) -> Result<Value, Box<dyn Error>>;
    async fn get_navaid_by_icao_code(self: &Self, icao: String) -> Result<Value, Box<dyn Error>>;
    async fn get_navaid_by_id(self: &Self, id: i64) -> Result<Value, Box<dyn Error>>;
    async fn search_navaid(
        self: &Self,
        search: Option<String>,
        page: Option<u32>,
        country: Option<String>,
        navaid_type: Option<String>,
    ) -> Result<Value, Box<dyn Error>>;
    async fn search_airport(
        self: &Self,
        search: Option<String>,
        page: Option<u32>,
        country: Option<String>,
        airport_type: Option<String>,
    ) -> Result<Value, Box<dyn Error>>;
}
