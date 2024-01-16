pub mod db;
pub mod messages;
pub mod navdata;
pub mod security;

use log::info;

pub fn register_routes(cfg: &mut actix_web::web::ServiceConfig) {
    navdata::airport::register_routes(cfg);
    navdata::navaid::register_routes(cfg);

    info!("Routes loaded");
}
