use crate::app::{
    db::AppState,
    navdata::model::{get_airport_by_icao_code, search_airport},
};
use actix_web::{get, web, HttpResponse, Responder};
use log::{error, info};
use serde::Deserialize;
use serde_json::json;

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(airport_by_icao_code);
    cfg.service(airport);

    info!("airports routes loaded");
}

#[derive(Deserialize)]
struct FormData {
    page: Option<u32>,
    search: Option<String>,
    country: Option<String>,
    airport_type: Option<String>,
}

#[get("/airport")]
async fn airport(param: web::Query<FormData>, app_state: web::Data<AppState>) -> impl Responder {
    info!("Request received : /airport");
    let data = search_airport(
        param.search.clone(),
        param.page,
        param.country.clone(),
        param.airport_type.clone(),
        app_state,
    )
    .await;
    match data {
        Ok(data) => HttpResponse::Ok().json(json!({"status": "success", "airports" : data})),
        Err(err) => {
            error!("Error while answering request /airport: {}", err);
            HttpResponse::Ok().json(json!({"status": "error"}))
        }
    }
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
