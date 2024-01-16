use crate::app::{db::AppState, messages::ERROR_SQLITE_ACCESS};
use actix_web::{get, web, HttpResponse, Responder};
use log::{error, info};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlite::State;
use std::error::Error;

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(airport_by_icao_code);
    cfg.service(airport);

    info!("airports routes loaded");
}

#[derive(Deserialize)]
struct FormData {
    page: Option<u32>,
    search: Option<String>,
}

async fn search_airport(
    search: Option<String>,
    _page: Option<u32>,
    app_state: web::Data<AppState>,
) -> Result<Value, Box<dyn Error>> {
    let codes = {
        let con = app_state
            .sqlite_connection
            .lock()
            .expect(ERROR_SQLITE_ACCESS);

        let mut statement = match search {
            Some(search) => {
                let query = "SELECT icao_code FROM airports WHERE icao_code LIKE '%' || ? || '%' OR name LIKE '%' || ? || '%' OR municipality LIKE '%' || ? || '%' OR iata_code LIKE '%' || ? || '%' LIMIT 100";
                let mut s = con.prepare(query)?;
                s.bind((1, search.as_str()))?;
                s.bind((2, search.as_str()))?;
                s.bind((3, search.as_str()))?;
                s
            }
            None => {
                let query = "SELECT * FROM airports LIMIT 100";
                con.prepare(query)?
            }
        };

        let mut codes = vec![];
        while let Ok(State::Row) = statement.next() {
            let icao_code = statement.read::<String, _>("icao_code")?;
            codes.push(icao_code);
        }
        codes
    };

    let mut data = json!([]);
    for code in codes {
        let airport_data = get_airport_by_icao_code(code, app_state.clone()).await?;
        data.as_array_mut().unwrap().push(airport_data);
    }

    Ok(data)
}

#[get("/airport")]
async fn airport(param: web::Query<FormData>, app_state: web::Data<AppState>) -> impl Responder {
    info!("Request received : /airport");
    let data = search_airport(param.search.clone(), param.page, app_state).await;
    match data {
        Ok(data) => HttpResponse::Ok().json(json!({"status": "success", "navaid" : data})),
        Err(err) => {
            error!("Error while answering request /airport: {}", err);
            HttpResponse::Ok().json(json!({"status": "error"}))
        }
    }
}

async fn get_airport_by_icao_code(
    icao: String,
    app_state: web::Data<AppState>,
) -> Result<Value, Box<dyn Error>> {
    let query = "SELECT * FROM airports WHERE icao_code=?";

    let icao = icao.to_uppercase();

    let mut data = json!({});

    let con = app_state
        .sqlite_connection
        .lock()
        .expect(ERROR_SQLITE_ACCESS);
    let mut statement = con.prepare(query)?;
    statement.bind((1, icao.as_str()))?;

    while let Ok(State::Row) = statement.next() {
        for column_name in statement.column_names() {
            data.as_object_mut().unwrap().insert(
                column_name.clone(),
                json!(statement.read::<String, _>(column_name.as_str())?),
            );
        }
        let mut freqs = json!([]);
        {
            let query = "SELECT type, description, frequency_mhz FROM airport_frequencies WHERE airport_icao_code=?";
            let mut freq_statement = con.prepare(query)?;
            freq_statement.bind((1, icao.as_str()))?;

            while let Ok(State::Row) = freq_statement.next() {
                let mut freq = json!({});
                for column_name in freq_statement.column_names() {
                    freq.as_object_mut().unwrap().insert(
                        column_name.clone(),
                        json!(freq_statement.read::<String, _>(column_name.as_str())?),
                    );
                }
                freqs.as_array_mut().unwrap().push(freq);
            }
        }
        data.as_object_mut()
            .unwrap()
            .insert(String::from("frequencies"), freqs);

        let mut runways = json!([]);
        {
            let query = "SELECT length_ft,width_ft,surface,lighted,closed,le_ident,le_latitude_deg,le_longitude_deg,le_elevation_ft,le_heading_degT,le_displaced_threshold_ft,he_ident,he_latitude_deg,he_longitude_deg,he_elevation_ft,he_heading_degT,he_displaced_threshold_ft FROM airport_runways WHERE airport_icao_code=?";
            let mut runway_statement = con.prepare(query)?;
            runway_statement.bind((1, icao.as_str()))?;

            while let Ok(State::Row) = runway_statement.next() {
                let mut runway = json!({});

                for column_name in runway_statement.column_names() {
                    runway.as_object_mut().unwrap().insert(
                        column_name.clone(),
                        json!(runway_statement.read::<String, _>(column_name.as_str())?),
                    );
                }

                runways.as_array_mut().unwrap().push(runway);
            }
        }
        data.as_object_mut()
            .unwrap()
            .insert(String::from("runways"), runways);
    }
    Ok(data)
}

#[get("/airport/{icao}")]
async fn airport_by_icao_code(
    icao: web::Path<String>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    info!("Request received : /airport/{}", icao);
    let data = get_airport_by_icao_code(icao.to_string(), app_state).await;
    match data {
        Ok(data) => HttpResponse::Ok().json(json!({"status": "success", "airport" : data})),
        Err(err) => {
            error!("Error while answering request /airport/{} : {}", icao, err);
            HttpResponse::Ok().json(json!({"status": "error"}))
        }
    }
}
