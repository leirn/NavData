use actix_web::web;
use log::{debug, info};
use serde_json::Value;
use sqlite::Connection;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Mutex;
use tokio::time::{sleep, Duration};

use crate::app::messages::{CSV_FORMAT_ERROR, ERROR_SQLITE_ACCESS, HTTP_USER_AGENT};

pub struct AppState {
    pub sqlite_connection: Mutex<Connection>,
}

pub fn create_tables(app_state: web::Data<AppState>) -> Result<(), Box<dyn Error>> {
    info!("Start database creation");
    let query = "
    CREATE TABLE IF NOT EXISTS data_last_update (
        file TEXT UNIQUE PRIMARY KEY NOT NULL,
        sha TEXT NOT NULL,
        date INTEGER
        );
    CREATE TABLE IF NOT EXISTS airports (
        id INTEGER UNIQUE,
        icao_code TEXT UNIQUE PRIMARY KEY NOT NULL,
        type TEXT,
        name TEXT,
        latitude_deg DECIMAL,
        longitude_deg DECIMAL,
        elevation_ft INTEGER,
        continent TEXT,
        iso_country TEXT,
        iso_region TEXT,
        municipality TEXT,
        scheduled_service TEXT,
        gps_code TEXT,
        iata_code TEXT,
        local_code TEXT,
        home_link TEXT,
        wikipedia_link TEXT,
        keywords TEXT
    );
    CREATE TABLE IF NOT EXISTS airport_frequencies (
        id INTEGER UNIQUE PRIMARY KEY NOT NULL,
        airport_ref INTEGER,
        airport_icao_code TEXT,
        type TEXT,
        description TEXT,
        frequency_mhz TEXT
    );
    CREATE TABLE IF NOT EXISTS airport_runways (
        id INTEGER UNIQUE PRIMARY KEY NOT NULL,
        airport_ref INTEGER,
        airport_icao_code TEXT,
        length_ft INTERGER,
        width_ft INTERGER,
        surface INTERGER,
        lighted INTERGER,
        closed INTERGER,
        le_ident INTERGER,
        le_latitude_deg DECIMAL,
        le_longitude_deg DECIMAL,
        le_elevation_ft INTERGER,
        le_heading_degT INTERGER,
        le_displaced_threshold_ft INTERGER,
        he_ident INTERGER,
        he_latitude_deg DECIMAL,
        he_longitude_deg DECIMAL,
        he_elevation_ft INTERGER,
        he_heading_degT INTERGER,
        he_displaced_threshold_ft INTERGER
    );
    CREATE TABLE IF NOT EXISTS navaids (
        id INTEGER UNIQUE PRIMARY KEY NOT NULL,
        filename TEXT NOT NULL,
        icao_code TEXT NOT NULL,
        name TEXT,
        type TEXT,
        frequency_khz INTEGER,
        latitude_deg DECIMAL,
        longitude_deg DECIMAL,
        elevation_ft INTEGER,
        iso_country TEXT,
        dme_frequency_khz INTEGER,
        dme_channel TEXT,
        dme_latitude_deg DECIMAL,
        dme_longitude_deg DECIMAL,
        dme_elevation_ft INTEGER,
        slaved_variation_deg INTEGER,
        magnetic_variation_deg INTEGER,
        usageType TEXT,
        power TEXT,
        associated_airport TEXT
    );
    ";
    app_state
        .sqlite_connection
        .lock()
        .expect(ERROR_SQLITE_ACCESS)
        .execute(query)?;
    info!("Database fully created");
    Ok(())
}

const BRANCH_API: &str =
    "https://api.github.com/repos/davidmegginson/ourairports-data/branches/main";
const TREE_API: &str = "https://api.github.com/repos/davidmegginson/ourairports-data/git/trees/";

const CSV_ROOT_URL: &str =
    "https://raw.githubusercontent.com/davidmegginson/ourairports-data/main/";
const AIRPORT_CSV: &str = "airports.csv";
const AIRPORT_FREQUENCY_CSV: &str = "airport-frequencies.csv";
const AIRPORT_RUNWAY_CSV: &str = "runways.csv";
const NAVAID_CSV: &str = "navaids.csv";

async fn load_airports(app_state: web::Data<AppState>) -> Result<(), Box<dyn Error>> {
    let result = reqwest::get(format!("{}{}", CSV_ROOT_URL, AIRPORT_CSV)).await?;
    let data = result.text().await?;
    let mut reader = csv::ReaderBuilder::new().from_reader(data.as_bytes());

    let con = app_state
        .sqlite_connection
        .lock()
        .expect(ERROR_SQLITE_ACCESS);

    let query = "DELETE FROM airports";
    con.execute(query)?;

    for result in reader.records() {
        let record = result?;

        let query = "INSERT INTO airports
            (id, icao_code, type, name, latitude_deg, longitude_deg, elevation_ft, continent, iso_region, iso_country, municipality, scheduled_service, gps_code, iata_code, local_code, home_link, wikipedia_link, keywords)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
        let mut statement = con.prepare(query)?;
        for i in 0..18 {
            statement.bind((i + 1, record.get(i).expect(CSV_FORMAT_ERROR)))?;
        }

        statement.next()?;
    }

    let query = "SELECT count(*) as count from airports";
    con.iterate(query, |result| {
        for &(_, value) in result.iter() {
            info!("{} airports loaded", value.unwrap());
        }
        true
    })?;

    Ok(())
}

async fn load_airport_frequencies(app_state: web::Data<AppState>) -> Result<(), Box<dyn Error>> {
    let result = reqwest::get(format!("{}{}", CSV_ROOT_URL, AIRPORT_FREQUENCY_CSV)).await?;
    let data = result.text().await?;
    let mut reader = csv::ReaderBuilder::new().from_reader(data.as_bytes());

    let con = app_state
        .sqlite_connection
        .lock()
        .expect(ERROR_SQLITE_ACCESS);

    let query = "DELETE FROM airport_frequencies";
    con.execute(query)?;

    for result in reader.records() {
        let record = result?;

        let query = "INSERT INTO airport_frequencies
            (id, airport_ref, airport_icao_code, type, description, frequency_mhz)
            VALUES (?, ?, ?, ?, ?, ?)";
        let mut statement = con.prepare(query)?;
        for i in 0..6 {
            statement.bind((i + 1, record.get(i).expect(CSV_FORMAT_ERROR)))?;
        }

        statement.next()?;
    }

    let query = "SELECT count(*) as count from airport_frequencies";
    con.iterate(query, |result| {
        for &(_, value) in result.iter() {
            info!("{} airport frequencies loaded", value.unwrap());
        }
        true
    })?;

    Ok(())
}

async fn load_airport_runways(app_state: web::Data<AppState>) -> Result<(), Box<dyn Error>> {
    let result = reqwest::get(format!("{}{}", CSV_ROOT_URL, AIRPORT_RUNWAY_CSV)).await?;
    let data = result.text().await?;
    let mut reader = csv::ReaderBuilder::new().from_reader(data.as_bytes());

    let con = app_state
        .sqlite_connection
        .lock()
        .expect(ERROR_SQLITE_ACCESS);

    let query = "DELETE FROM airport_runways";
    con.execute(query)?;

    for result in reader.records() {
        let record = result?;

        let query = "INSERT INTO airport_runways
            (id, airport_ref, airport_icao_code, length_ft,width_ft,surface,lighted,closed,le_ident,le_latitude_deg,le_longitude_deg,le_elevation_ft,le_heading_degT,le_displaced_threshold_ft,he_ident,he_latitude_deg,he_longitude_deg,he_elevation_ft,he_heading_degT,he_displaced_threshold_ft)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
        let mut statement = con.prepare(query)?;
        for i in 0..20 {
            statement.bind((i + 1, record.get(i).expect(CSV_FORMAT_ERROR)))?;
        }

        statement.next()?;
    }

    let query = "SELECT count(*) as count from airport_runways";
    con.iterate(query, |result| {
        for &(_, value) in result.iter() {
            info!("{} airport runways loaded", value.unwrap());
        }
        true
    })?;

    Ok(())
}

async fn load_navaids(app_state: web::Data<AppState>) -> Result<(), Box<dyn Error>> {
    let result = reqwest::get(format!("{}{}", CSV_ROOT_URL, NAVAID_CSV)).await?;
    let data = result.text().await?;
    let mut reader = csv::ReaderBuilder::new().from_reader(data.as_bytes());

    let con = app_state
        .sqlite_connection
        .lock()
        .expect(ERROR_SQLITE_ACCESS);

    let query = "DELETE FROM navaids";
    con.execute(query)?;

    for result in reader.records() {
        let record = result?;

        let query = "INSERT INTO navaids
            (id,filename,icao_code,name,type,frequency_khz,latitude_deg,longitude_deg,elevation_ft,iso_country,dme_frequency_khz,dme_channel,dme_latitude_deg,dme_longitude_deg,dme_elevation_ft,slaved_variation_deg,magnetic_variation_deg,usageType,power,associated_airport )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
        let mut statement = con.prepare(query)?;
        for i in 0..20 {
            statement.bind((i + 1, record.get(i).expect(CSV_FORMAT_ERROR)))?;
        }

        statement.next()?;
    }

    let query = "SELECT count(*) as count from navaids";
    con.iterate(query, |result| {
        for &(_, value) in result.iter() {
            info!("{} airport runways loaded", value.unwrap());
        }
        true
    })?;

    Ok(())
}

async fn get_list_of_sha() -> Result<HashMap<String, String>, Box<dyn Error>> {
    debug!("Looking for branch sha first");

    let client = reqwest::Client::builder()
        .user_agent(HTTP_USER_AGENT)
        .build()?;

    let result = client.get(BRANCH_API).send().await?;

    let mut shas = HashMap::new();

    let data = result.json::<Value>().await?;
    let branch_sha = data
        .get("commit")
        .unwrap()
        .get("sha")
        .unwrap()
        .as_str()
        .unwrap();

    debug!("Branch sha is {}", branch_sha);

    let result = client
        .get(format!("{}{}", TREE_API, branch_sha))
        .send()
        .await?;
    let data = result.json::<Value>().await?;
    for file in data.get("tree").unwrap().as_array().unwrap() {
        shas.insert(
            String::from(file.get("path").unwrap().as_str().unwrap()),
            String::from(file.get("sha").unwrap().as_str().unwrap()),
        );
    }

    Ok(shas)
}

/// Returns true if sha had been updated to database
fn check_and_store_sha(
    app_state: web::Data<AppState>,
    file: &str,
    sha: &String,
) -> Result<bool, Box<dyn Error>> {
    let con = app_state
        .sqlite_connection
        .lock()
        .expect(ERROR_SQLITE_ACCESS);

    let query = "SELECT count(*) as count FROM data_last_update WHERE file = ? AND sha = ?";

    let mut s = con.prepare(query)?;
    s.bind((1, file))?;
    s.bind((2, sha.as_str()))?;

    s.next()?;
    let count = s.read::<i64, _>("count")?;
    s.reset()?;

    if count == 0 {
        // sha in DB is different than the one provided, update database
        let query = "REPLACE INTO data_last_update (file, sha, date) VALUES (?, ?, unixepoch())";
        let mut s2 = con.prepare(query)?;
        s2.bind((1, file))?;
        s2.bind((2, sha.as_str()))?;
        s2.next()?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub async fn periodical_update(app_state: web::Data<AppState>) -> Result<(), Box<dyn Error>> {
    loop {
        info!("Awake ! reloading data");

        let shas = get_list_of_sha().await?;

        match check_and_store_sha(
            app_state.clone(),
            AIRPORT_CSV,
            shas.get(AIRPORT_CSV).unwrap(),
        )? {
            true => {
                load_airports(app_state.clone()).await?;
            }
            false => (),
        }

        match check_and_store_sha(
            app_state.clone(),
            AIRPORT_FREQUENCY_CSV,
            shas.get(AIRPORT_FREQUENCY_CSV).unwrap(),
        )? {
            true => {
                load_airport_frequencies(app_state.clone()).await?;
            }
            false => (),
        }

        match check_and_store_sha(
            app_state.clone(),
            AIRPORT_RUNWAY_CSV,
            shas.get(AIRPORT_RUNWAY_CSV).unwrap(),
        )? {
            true => {
                load_airport_runways(app_state.clone()).await?;
            }
            false => (),
        }

        match check_and_store_sha(app_state.clone(), NAVAID_CSV, shas.get(NAVAID_CSV).unwrap())? {
            true => {
                load_navaids(app_state.clone()).await?;
            }
            false => (),
        }

        info!("Database fully reloaded");
        let _delay = sleep(Duration::from_secs(30)).await;
    }
}
