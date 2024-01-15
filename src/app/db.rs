use actix_web::web;
use log::info;
use sqlite::Connection;
use std::error::Error;
use std::sync::Mutex;

pub struct AppState {
    pub sqlite_connection: Mutex<Connection>,
}

pub fn create_tables(app_state: web::Data<AppState>) {
    let query = "
    CREATE TABLE IF NOT EXISTS data_last_update (
        file TEXT UNIQUE PRIMARY KEY NOT NULL,
        sha TEXT NOT NULL,
        date INTEGER
        );
    CREATE TABLE IF NOT EXISTS airports (
        id INTEGER UNIQUE,
        ident TEXT UNIQUE PRIMARY KEY NOT NULL,
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
    ";
    app_state
        .sqlite_connection
        .lock()
        .unwrap()
        .execute(query)
        .unwrap();
}

const BRANCH_API: &str =
    "https://api.github.com/repos/davidmegginson/ourairports-data/branches/main";
const TREE_API: &str = "https://api.github.com/repos/davidmegginson/ourairports-data/git/trees/";

const AIRPORT_CSV: &str =
    "https://raw.githubusercontent.com/davidmegginson/ourairports-data/main/airports.csv";

pub async fn load_database(app_state: web::Data<AppState>) -> Result<(), Box<dyn Error>> {
    let result = reqwest::get(AIRPORT_CSV).await;
    let result = result.unwrap();
    let data = result.text().await;
    let data = data.unwrap();
    let mut reader = csv::ReaderBuilder::new().from_reader(data.as_bytes());

    let con = app_state.sqlite_connection.lock().unwrap();

    for result in reader.records() {
        let record = result?;

        let query = "INSERT INTO airports
            (id, ident, type, name, latitude_deg, longitude_deg, elevation_ft, continent, iso_region, iso_country, municipality, scheduled_service, gps_code, iata_code, local_code, home_link, wikipedia_link, keywords)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
        let mut statement = con.prepare(query)?;
        statement.bind((1, record.get(0).unwrap()))?;
        statement.bind((2, record.get(1).unwrap()))?;
        statement.bind((3, record.get(2).unwrap()))?;
        statement.bind((4, record.get(3).unwrap()))?;
        statement.bind((5, record.get(4).unwrap()))?;
        statement.bind((6, record.get(5).unwrap()))?;
        statement.bind((7, record.get(6).unwrap()))?;
        statement.bind((8, record.get(7).unwrap()))?;
        statement.bind((9, record.get(8).unwrap()))?;
        statement.bind((10, record.get(9).unwrap()))?;
        statement.bind((11, record.get(10).unwrap()))?;
        statement.bind((12, record.get(11).unwrap()))?;
        statement.bind((13, record.get(12).unwrap()))?;
        statement.bind((14, record.get(13).unwrap()))?;
        statement.bind((15, record.get(14).unwrap()))?;
        statement.bind((16, record.get(15).unwrap()))?;
        statement.bind((17, record.get(16).unwrap()))?;
        statement.bind((18, record.get(17).unwrap()))?;

        statement.next()?;
    }

    let query = "SELECT count(*) as count from airports";
    con.iterate(query, |result| {
        for &(_, value) in result.iter() {
            info!("{} airports loaded", value.unwrap());
        }
        true
    })
    .unwrap();

    Ok(())
}
