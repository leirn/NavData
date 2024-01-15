mod app;
use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use app::db::periodical_update;

use crate::app::db::{create_tables, load_database, AppState};
use sqlite;
use std::env;
use std::sync::Mutex;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let host = env::var("HOST").expect("$HOST is not set");

    let port = env::var("PORT")
        .expect("$PORT is not set")
        .parse()
        .expect("$PORT cannot be converted to uint_16");

    let connection = sqlite::open(":memory:").unwrap();

    let app_state = web::Data::new(AppState {
        sqlite_connection: Mutex::new(connection),
    });

    create_tables(app_state.clone());
    load_database(app_state.clone()).await.unwrap();

    actix_rt::spawn(periodical_update(app_state.clone()));

    HttpServer::new(move || {
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .wrap(Cors::permissive().supports_credentials())
            .app_data(app_state.clone())
            .configure(app::register_routes)
    })
    .bind((host, port))?
    .run()
    .await
}
