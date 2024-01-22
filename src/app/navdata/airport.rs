use crate::app::db::AppState;
use actix_web::{get, web, HttpResponse, Responder};
use log::{error, info};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(airport_by_icao_code);
    cfg.service(airport);

    info!("airports routes loaded");
}

#[derive(Deserialize)]
struct FormData {
    page: Option<u64>,
    search: Option<String>,
    country: Option<String>,
    airport_type: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

#[get("/airport")]
async fn airport(param: web::Query<FormData>, app_state: web::Data<AppState>) -> impl Responder {
    info!("Request received : /airport");
    let data = app_state
        .database
        .search_airport(
            param.search.clone(),
            param.page,
            param.country.clone(),
            param.airport_type.clone(),
            param.latitude,
            param.longitude,
        )
        .await;
    match data {
        Ok(data) => HttpResponse::Ok().json(json!({"status": "success", "airports" : data})),
        Err(err) => {
            let error_id = Uuid::new_v4();
            error!(
                "[{}] Error while answering request /airport : {}",
                error_id, err
            );
            HttpResponse::Ok().json(json!({"status": "error", "description" : format!("Error {} : contact your administrator", error_id)}))
        }
    }
}

#[get("/airport/{icao}")]
async fn airport_by_icao_code(
    icao: web::Path<String>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    info!("Request received : /airport/{}", icao);

    if icao.len() != 4 {
        return HttpResponse::Ok().json(
            json!({"status": "error", "description":"Airport ICAO codes must be 4 letter long"}),
        );
    }

    let data = app_state
        .database
        .get_airport_by_icao_code(icao.to_string())
        .await;
    match data {
        Ok(data) => match data {
            Some(data) => {
                HttpResponse::Ok().json(json!({"status": "success", "airport" : data, "count" : 1}))
            }
            None => HttpResponse::Ok().json(json!({"status": "success", "count" : 0})),
        },
        Err(err) => {
            let error_id = Uuid::new_v4();
            error!(
                "[{}] Error while answering request /airport/{} : {}",
                error_id, icao, err
            );
            HttpResponse::Ok().json(json!({"status": "error", "description" : format!("Error {} : contact your administrator", error_id)}))
        }
    }
}
