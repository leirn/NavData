use actix_web::{get, web, HttpResponse, Responder};
use log::{error, info};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlite::State;
use std::error::Error;

use crate::app::{db::AppState, messages::ERROR_SQLITE_ACCESS};

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(navaid);
    cfg.service(navaid_by_icao_code);

    info!("navaids routes loaded");
}

#[derive(Deserialize)]
struct FormData {
    page: Option<u32>,
    search: Option<String>,
}

async fn search_navaid(
    search: Option<String>,
    _page: Option<u32>,
    app_state: web::Data<AppState>,
) -> Result<Value, Box<dyn Error>> {
    let con = app_state
        .sqlite_connection
        .lock()
        .expect(ERROR_SQLITE_ACCESS);

    let mut statement = match search {
        Some(search) => {
            let query = "SELECT * FROM navaids WHERE icao_code LIKE '%' || ? || '%' OR name LIKE '%' || ? || '%' OR associated_airport LIKE '%' || ? || '%' LIMIT 100";
            let mut s = con.prepare(query)?;
            s.bind((1, search.as_str()))?;
            s.bind((2, search.as_str()))?;
            s.bind((3, search.as_str()))?;
            s
        }
        None => {
            let query = "SELECT * FROM navaids LIMIT 100";
            con.prepare(query)?
        }
    };

    let mut data = json!([]);

    while let Ok(State::Row) = statement.next() {
        let mut navaid_data = json!({});

        for column_name in statement.column_names() {
            navaid_data.as_object_mut().unwrap().insert(
                column_name.clone(),
                json!(statement.read::<String, _>(column_name.as_str())?),
            );
        }
        data.as_array_mut().unwrap().push(navaid_data);
    }
    Ok(data)
}

#[get("/navaid")]
async fn navaid(param: web::Query<FormData>, app_state: web::Data<AppState>) -> impl Responder {
    info!("Request received : /navaid");
    let data = search_navaid(param.search.clone(), param.page, app_state).await;
    match data {
        Ok(data) => HttpResponse::Ok().json(json!({"status": "success", "navaid" : data})),
        Err(err) => {
            error!("Error while answering request /navaid: {}", err);
            HttpResponse::Ok().json(json!({"status": "error"}))
        }
    }
}

async fn get_navaid_by_icao_code(
    icao: String,
    app_state: web::Data<AppState>,
) -> Result<Value, Box<dyn Error>> {
    let query = "SELECT * FROM navaids WHERE icao_code=?";

    let mut data = json!([]);

    let con = app_state
        .sqlite_connection
        .lock()
        .expect(ERROR_SQLITE_ACCESS);
    let mut statement = con.prepare(query)?;
    statement.bind((1, icao.as_str()))?;

    while let Ok(State::Row) = statement.next() {
        let mut navaid_data = json!({});

        for column_name in statement.column_names() {
            navaid_data.as_object_mut().unwrap().insert(
                column_name.clone(),
                json!(statement.read::<String, _>(column_name.as_str())?),
            );
        }
        data.as_array_mut().unwrap().push(navaid_data);
    }
    Ok(data)
}

#[get("/navaid/{icao}")]
async fn navaid_by_icao_code(
    icao: web::Path<String>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    info!("Request received : /navaid/{}", icao);
    let data = get_navaid_by_icao_code(icao.to_string(), app_state).await;
    match data {
        Ok(data) => HttpResponse::Ok().json(json!({"status": "success", "navaid" : data})),
        Err(err) => {
            error!("Error while answering request /navaid/{} : {}", icao, err);
            HttpResponse::Ok().json(json!({"status": "error"}))
        }
    }
}
