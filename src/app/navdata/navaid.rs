use crate::app::db::AppState;
use actix_web::{get, web, HttpResponse, Responder};
use log::{error, info};
use serde::Deserialize;
use serde_json::json;

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(navaid);
    cfg.service(navaid_by_icao_code);

    info!("navaids routes loaded");
}

#[derive(Deserialize)]
struct FormData {
    page: Option<u32>,
    search: Option<String>,
    country: Option<String>,
    navaid_type: Option<String>,
}

#[get("/navaid")]
async fn navaid(param: web::Query<FormData>, app_state: web::Data<AppState>) -> impl Responder {
    info!("Request received : /navaid");
    let data = app_state
        .database
        .search_navaid(
            param.search.clone(),
            param.page,
            param.country.clone(),
            param.navaid_type.clone(),
        )
        .await;
    match data {
        Ok(data) => HttpResponse::Ok().json(json!({"status": "success", "navaid" : data})),
        Err(err) => {
            error!("Error while answering request /navaid: {}", err);
            HttpResponse::Ok().json(json!({"status": "error"}))
        }
    }
}

#[get("/navaid/{icao}")]
async fn navaid_by_icao_code(
    icao: web::Path<String>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    info!("Request received : /navaid/{}", icao);
    let data = app_state
        .database
        .get_navaid_by_icao_code(icao.to_string())
        .await;
    match data {
        Ok(data) => HttpResponse::Ok().json(json!({"status": "success", "navaid" : data})),
        Err(err) => {
            error!("Error while answering request /navaid/{} : {}", icao, err);
            HttpResponse::Ok().json(json!({"status": "error"}))
        }
    }
}
