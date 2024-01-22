use serde::Deserialize;

use super::db::BackendType;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub http: HttpConfig,
    pub security: SecurityConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Deserialize, Default)]
pub struct HttpConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Default)]
pub struct SecurityConfig {
    pub auth_tokens: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct DatabaseConfig {
    pub backend: BackendType,
    pub path: Option<String>,
}
