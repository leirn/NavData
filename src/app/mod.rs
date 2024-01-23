pub mod config;
pub mod db;
pub mod messages;
pub mod routes;
pub mod security;

use log::info;

pub fn register_routes(cfg: &mut actix_web::web::ServiceConfig) {
    routes::airport::register_routes(cfg);
    routes::navaid::register_routes(cfg);

    info!("Routes loaded");
}
