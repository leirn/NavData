use actix_web::{get, web, HttpResponse, Responder};
use log::info;
use serde_json::json;
use sqlite::State;

use crate::app::db::AppState;

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(airport);

    info!("airports routes loaded");
}

#[get("/airport/{icao}")]
async fn airport(icao: web::Path<String>, app_state: web::Data<AppState>) -> impl Responder {
    let query = "SELECT * FROM airports WHERE ident=?";

    let mut data = json!({});

    let con = app_state.sqlite_connection.lock().unwrap();
    let mut statement = con.prepare(query).unwrap();
    statement.bind((1, icao.as_str())).unwrap();

    while let Ok(State::Row) = statement.next() {
        for column_name in statement.column_names() {
            data.as_object_mut().unwrap().insert(
                column_name.clone(),
                json!(statement.read::<String, _>(column_name.as_str()).unwrap()),
            );
        }
    }

    HttpResponse::Ok().json(json!({"status": "success", "airport" : data}))
}
