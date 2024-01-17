mod app;
use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use app::db::periodical_update;
use app::messages::*;
use app::security::simple_token::SimpleToken;

use crate::app::db::{create_tables, AppState};
use sqlite;
use std::env;
use std::sync::Mutex;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let host = env::var(PARAM_HOST).unwrap_or(String::from(DEFAULT_HOST));
    let database_path = env::var(PARAM_DATABASE_PATH).unwrap_or(String::from(DEFAULT_DATABASE));

    let port = env::var(PARAM_PORT)
        .unwrap_or(String::from(DEFAULT_PORT))
        .parse()
        .expect(PORT_ERROR);

    let connection = sqlite::open(database_path).expect(ERROR_SQLITE_ACCESS);

    let app_state = web::Data::new(AppState {
        sqlite_connection: Mutex::new(connection),
    });

    create_tables(app_state.clone()).unwrap();

    actix_rt::spawn(periodical_update(app_state.clone()));

    HttpServer::new(move || {
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .wrap(Cors::permissive().supports_credentials())
            .wrap(SimpleToken)
            .app_data(app_state.clone())
            .configure(app::register_routes)
    })
    .bind((host, port))?
    .run()
    .await
}
