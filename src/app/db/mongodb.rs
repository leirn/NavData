use super::DatabaseBackend;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::error::Error;

pub struct MongoDbBackend {}

impl MongoDbBackend {
    pub fn new() -> MongoDbBackend {
        MongoDbBackend {}
    }
}
#[async_trait]
impl DatabaseBackend for MongoDbBackend {
    async fn periodical_update(&self) {}
    async fn get_airport_by_icao_code(&self, icao: String) -> Result<Value, Box<dyn Error>> {
        Ok(json!({}))
    }
    async fn get_navaid_by_icao_code(&self, icao: String) -> Result<Value, Box<dyn Error>> {
        Ok(json!({}))
    }
    async fn get_navaid_by_id(&self, id: i64) -> Result<Value, Box<dyn Error>> {
        Ok(json!({}))
    }
    async fn search_navaid(
        &self,
        search: Option<String>,
        page: Option<u32>,
        country: Option<String>,
        navaid_type: Option<String>,
    ) -> Result<Value, Box<dyn Error>> {
        Ok(json!({}))
    }
    async fn search_airport(
        &self,
        search: Option<String>,
        page: Option<u32>,
        country: Option<String>,
        airport_type: Option<String>,
    ) -> Result<Value, Box<dyn Error>> {
        Ok(json!({}))
    }
}
