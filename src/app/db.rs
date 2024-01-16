use actix_web::web;
use log::info;
use sqlite::Connection;
use std::error::Error;
use std::sync::Mutex;
use tokio::time::{sleep, Duration};

use crate::app::messages::{CSV_FORMAT_ERROR, ERROR_SQLITE_ACCESS};

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

const AIRPORT_CSV: &str =
    "https://raw.githubusercontent.com/davidmegginson/ourairports-data/main/airports.csv";
const AIRPORT_FREQUENCY_CSV: &str =
        "https://raw.githubusercontent.com/davidmegginson/ourairports-data/main/airport-frequencies.csv";
const AIRPORT_RUNWAY_CSV: &str =
    "https://raw.githubusercontent.com/davidmegginson/ourairports-data/main/runways.csv";
const NAVAID_CSV: &str =
    "https://raw.githubusercontent.com/davidmegginson/ourairports-data/main/navaids.csv";

pub async fn load_database(app_state: web::Data<AppState>) -> Result<(), Box<dyn Error>> {
    load_airports(app_state.clone()).await?;
    load_airport_frequencies(app_state.clone()).await?;
    load_airport_runways(app_state.clone()).await?;
    load_navaids(app_state.clone()).await?;
    Ok(())
}

async fn load_airports(app_state: web::Data<AppState>) -> Result<(), Box<dyn Error>> {
    let result = reqwest::get(AIRPORT_CSV).await?;
    let data = result.text().await?;
    let mut reader = csv::ReaderBuilder::new().from_reader(data.as_bytes());

    let con = app_state
        .sqlite_connection
        .lock()
        .expect(ERROR_SQLITE_ACCESS);

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
    let result = reqwest::get(AIRPORT_FREQUENCY_CSV).await?;
    let data = result.text().await?;
    let mut reader = csv::ReaderBuilder::new().from_reader(data.as_bytes());

    let con = app_state
        .sqlite_connection
        .lock()
        .expect(ERROR_SQLITE_ACCESS);

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
    let result = reqwest::get(AIRPORT_RUNWAY_CSV).await?;
    let data = result.text().await?;
    let mut reader = csv::ReaderBuilder::new().from_reader(data.as_bytes());

    let con = app_state
        .sqlite_connection
        .lock()
        .expect(ERROR_SQLITE_ACCESS);

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
    let result = reqwest::get(NAVAID_CSV).await?;
    let data = result.text().await?;
    let mut reader = csv::ReaderBuilder::new().from_reader(data.as_bytes());

    let con = app_state
        .sqlite_connection
        .lock()
        .expect(ERROR_SQLITE_ACCESS);

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

pub async fn periodical_update(app_state: web::Data<AppState>) -> Result<(), Box<dyn Error>> {
    loop {
        let _delay = sleep(Duration::from_secs(86400)).await;
        println!("Awake ! reloading data");
        let query = "DELETE FROM airports;
        DELETE FROM airport_frequencies;
        DELETE FROM airport_runways;
        DELETE FROM navaids";
        app_state
            .sqlite_connection
            .lock()
            .expect(ERROR_SQLITE_ACCESS)
            .execute(query)?;
        info!("Database fully cleaned");
        load_database(app_state.clone()).await?;
        info!("Database relaoded");
    }
}
