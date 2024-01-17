use actix_web::web;
use serde_json::{json, Value};
use sqlite::State;
use std::error::Error;

use crate::app::db::AppState;
use crate::app::messages::ERROR_SQLITE_ACCESS;

pub async fn get_airport_by_icao_code(
    icao: String,
    app_state: web::Data<AppState>,
) -> Result<Value, Box<dyn Error>> {
    let query = "SELECT * FROM airports WHERE icao_code=?";
    let icao = icao.to_uppercase();

    let mut data = json!({});
    {
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
    }
    let mut navaids = json!([]);
    {
        let mut ids = vec![];
        {
            let con = app_state
                .sqlite_connection
                .lock()
                .expect(ERROR_SQLITE_ACCESS);
            let query = "SELECT id FROM navaids WHERE associated_airport=?";
            let mut navaid_statement = con.prepare(query)?;
            navaid_statement.bind((1, icao.as_str()))?;

            while let Ok(State::Row) = navaid_statement.next() {
                ids.push(navaid_statement.read::<i64, _>("id")?);
            }
        }

        for id in ids {
            let navaid = get_navaid_by_id(id, app_state.clone()).await?;
            navaids.as_array_mut().unwrap().push(navaid);
        }
    }
    data.as_object_mut()
        .unwrap()
        .insert(String::from("navaids"), navaids);
    Ok(data)
}

pub async fn get_navaid_by_icao_code(
    icao: String,
    app_state: web::Data<AppState>,
) -> Result<Value, Box<dyn Error>> {
    let query = "SELECT * FROM navaids WHERE icao_code=?";

    let icao = icao.to_uppercase();

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

pub async fn get_navaid_by_id(
    id: i64,
    app_state: web::Data<AppState>,
) -> Result<Value, Box<dyn Error>> {
    let query = "SELECT * FROM navaids WHERE id=?";

    let con = app_state
        .sqlite_connection
        .lock()
        .expect(ERROR_SQLITE_ACCESS);
    let mut statement = con.prepare(query)?;
    statement.bind((1, id))?;
    statement.next()?;
    let mut navaid_data = json!({});

    for column_name in statement.column_names() {
        navaid_data.as_object_mut().unwrap().insert(
            column_name.clone(),
            json!(statement.read::<String, _>(column_name.as_str())?),
        );
    }
    Ok(navaid_data)
}

pub async fn search_navaid(
    search: Option<String>,
    _page: Option<u32>,
    country: Option<String>,
    app_state: web::Data<AppState>,
) -> Result<Value, Box<dyn Error>> {
    let con = app_state
        .sqlite_connection
        .lock()
        .expect(ERROR_SQLITE_ACCESS);

    // First build the query
    let mut query = "SELECT * FROM navaids".to_owned();
    let mut is_first = true;
    if country.is_some() {
        query.push_str(" WHERE iso_country = ?");
        is_first = false;
    }
    if search.is_some() {
        if is_first {
            query.push_str(" WHERE");
        } else {
            query.push_str(" AND");
        }
        query.push_str(" (icao_code LIKE '%' || ? || '%' OR name LIKE '%' || ? || '%' OR associated_airport LIKE '%' || ? || '%')");
    }
    query.push_str(" LIMIT 100");

    // Build and fill the statement
    let mut statement = con.prepare(query)?;
    let mut index = 1;
    if country.is_some() {
        let country_param = country.unwrap().to_uppercase();
        statement.bind((index, country_param.as_str()))?;
        index += 1;
    }
    if search.is_some() {
        let search = search.unwrap();
        let search_str = search.as_str();
        statement.bind((index, search_str))?;
        statement.bind((index + 1, search_str))?;
        statement.bind((index + 2, search_str))?;
        //index += 3;
    }

    // Execute statement and get the results
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

pub async fn search_airport(
    search: Option<String>,
    _page: Option<u32>,
    country: Option<String>,
    app_state: web::Data<AppState>,
) -> Result<Value, Box<dyn Error>> {
    let codes = {
        let con = app_state
            .sqlite_connection
            .lock()
            .expect(ERROR_SQLITE_ACCESS);

        // First build the query
        let mut query = "SELECT icao_code FROM airports".to_owned();
        let mut is_first = true;
        if country.is_some() {
            query.push_str(" WHERE iso_country = ?");
            is_first = false;
        }
        if search.is_some() {
            if is_first {
                query.push_str(" WHERE");
            } else {
                query.push_str(" AND");
            }
            query.push_str(" (icao_code LIKE '%' || ? || '%' OR name LIKE '%' || ? || '%' OR municipality LIKE '%' || ? || '%' OR iata_code LIKE '%' || ? || '%')");
        }
        query.push_str(" LIMIT 100");

        // Build and fill the statement
        let mut statement = con.prepare(query)?;
        let mut index = 1;
        if country.is_some() {
            let country_param = country.unwrap().to_uppercase();
            statement.bind((index, country_param.as_str()))?;
            index += 1;
        }
        if search.is_some() {
            let search = search.unwrap();
            let search_str = search.as_str();
            statement.bind((index, search_str))?;
            statement.bind((index + 1, search_str))?;
            statement.bind((index + 2, search_str))?;
            //index += 3;
        }

        // Execute statement and get the results
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
