use crate::app::db::AppState;
use actix_web::{get, web, HttpResponse, Responder};
use log::{error, info};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

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
    latitude: Option<f64>,
    longitude: Option<f64>,
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
            param.latitude,
            param.longitude,
        )
        .await;
    match data {
        Ok(data) => HttpResponse::Ok().json(json!({"status": "success", "navaid" : data})),
        Err(err) => {
            let error_id = Uuid::new_v4();
            error!(
                "[{}] Error while answering request /navaid : {}",
                error_id, err
            );
            HttpResponse::Ok().json(json!({"status": "error", "description" : format!("Error {} : contact your administrator", error_id)}))
        }
    }
}

#[get("/navaid/{icao}")]
async fn navaid_by_icao_code(
    icao: web::Path<String>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    info!("Request received : /navaid/{}", icao);

    if icao.len() != 3 {
        return HttpResponse::Ok().json(
            json!({"status": "error", "description":"Navaid ICAO codes must be 3 letter long"}),
        );
    }

    let data = app_state
        .database
        .get_navaids_by_icao_code(icao.to_string())
        .await;
    match data {
        Ok(data) => HttpResponse::Ok().json(json!({"status": "success", "navaid" : data})),
        Err(err) => {
            let error_id = Uuid::new_v4();
            error!(
                "[{}] Error while answering request /navaid/{} : {}",
                error_id, icao, err
            );
            HttpResponse::Ok().json(json!({"status": "error", "description" : format!("Error {} : contact your administrator", error_id)}))
        }
    }
}
