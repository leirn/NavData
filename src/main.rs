mod app;
use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use app::config::Config;
use app::db::{periodical_update, AppState, BackendType, DatabaseBackend};
use app::security::simple_token::SimpleToken;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// HTTP Server host
    #[arg(short, long, default_value = "127.0.0.1")]
    address: String,

    /// HTTP Server port
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// Database backend sqlite
    #[arg(short, long, default_value_t = true)]
    sqlite: bool,

    /// Database backend mongodb
    #[arg(short, long, default_value_t = false)]
    mongodb: bool,

    /// Batabase path. Use ":memory:" for in memory database for
    #[arg(short, long, default_value = ":memory:")]
    db_path: String,

    /// loglevel. 0 for error, 1 for warn, 2 for info, 3 for debug
    #[arg(short, long, default_value_t = 2)]
    loglevel: u8,

    /// path to YAML config file
    #[arg(short, long)]
    config: PathBuf,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let args = Args::parse();

    let mut config = Config::default();
    if args.config.exists() {
        let f = std::fs::File::open(args.config).expect("Could not open file.");
        config = serde_yaml::from_reader(f).expect("Could not read values.");
    } else {
        config.http.host = args.address.clone();
        config.http.port = args.port;

        config.database.backend = {
            if args.mongodb {
                BackendType::MONGODB
            } else {
                BackendType::SQLITE
            }
        };
        config.database.path = Some(args.db_path.clone());
    }
    let backend = DatabaseBackend::new(
        config.database.backend,
        config
            .database
            .path
            .clone()
            .unwrap_or(":memory:".to_string()),
    )
    .await;

    let host = config.http.host.clone();
    let port = config.http.port;

    let app_state: web::Data<AppState> = web::Data::new(AppState {
        database: backend,
        config: config,
    });

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
