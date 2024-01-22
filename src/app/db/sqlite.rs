use crate::app::messages::{CSV_FORMAT_ERROR, ERROR_SQLITE_ACCESS, HTTP_USER_AGENT};
use ::sqlite::Connection;
use log::{debug, info};
use serde_json::Value;
use sqlite::State;
use std::error::Error;
use std::str::FromStr;
use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};
use tokio::time::{sleep, Duration};

use super::{
    Airport, AirportType, Frequency, FrequencyType, LocationPoint, LocationType, Navaid,
    NavaidType, Runway,
};

pub struct SqliteBackend {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteBackend {
    pub fn new(path: String) -> SqliteBackend {
        // Init DB first
        let connection = sqlite::open(path.clone()).expect(ERROR_SQLITE_ACCESS);

        let s = SqliteBackend {
            connection: Arc::new(Mutex::new(connection)),
        };
        s.create_tables().unwrap();
        s
    }
    pub fn create_tables(&self) -> Result<(), Box<dyn Error>> {
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
        CREATE INDEX idx_airports_name ON airports (name);
        CREATE INDEX idx_airports_municipality ON airports (municipality);
        CREATE INDEX idx_airports_iata_code ON airports (iata_code);
        CREATE INDEX idx_airports_iso_country ON airports (iso_country);
        CREATE INDEX idx_airports_type ON airports (type);
        CREATE TABLE IF NOT EXISTS airport_frequencies (
            id INTEGER UNIQUE PRIMARY KEY NOT NULL,
            airport_ref INTEGER,
            airport_icao_code TEXT,
            type TEXT,
            description TEXT,
            frequency_mhz DECIMAL
        );
        CREATE TABLE IF NOT EXISTS airport_runways (
            id INTEGER UNIQUE PRIMARY KEY NOT NULL,
            airport_ref INTEGER,
            airport_icao_code TEXT,
            length_ft INTEGER,
            width_ft INTEGER,
            surface INTEGER,
            lighted INTEGER,
            closed INTEGER,
            le_ident INTEGER,
            le_latitude_deg DECIMAL,
            le_longitude_deg DECIMAL,
            le_elevation_ft INTEGER,
            le_heading_degT INTEGER,
            le_displaced_threshold_ft INTEGER,
            he_ident INTEGER,
            he_latitude_deg DECIMAL,
            he_longitude_deg DECIMAL,
            he_elevation_ft INTEGER,
            he_heading_degT INTEGER,
            he_displaced_threshold_ft INTEGER
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
        CREATE INDEX idx_navaids_name ON navaids (name);
        CREATE INDEX idx_navaids_filename ON navaids (filename);
        CREATE INDEX idx_navaids_associated_airport ON navaids (associated_airport);
        CREATE INDEX idx_navaids_type ON navaids (type);
        CREATE INDEX idx_navaids_iso_country ON navaids (iso_country);
        ";
        self.connection
            .lock()
            .expect(ERROR_SQLITE_ACCESS)
            .execute(query)?;
        info!("Database fully created");
        Ok(())
    }

    pub async fn load_airports(&self) -> Result<(), Box<dyn Error>> {
        let result = reqwest::get(format!("{}{}", super::CSV_ROOT_URL, super::AIRPORT_CSV)).await?;
        let data = result.text().await?;
        let mut reader = csv::ReaderBuilder::new().from_reader(data.as_bytes());

        let con = self.connection.lock().expect(ERROR_SQLITE_ACCESS);

        let query = "DELETE FROM airports";
        con.execute(query)?;

        for result in reader.records() {
            let record = result?;

            let query = "INSERT INTO airports
                (id, icao_code, type, name, latitude_deg, longitude_deg, elevation_ft, continent, iso_country, iso_region, municipality, scheduled_service, gps_code, iata_code, local_code, home_link, wikipedia_link, keywords)
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

    pub async fn load_airport_frequencies(&self) -> Result<(), Box<dyn Error>> {
        let result = reqwest::get(format!(
            "{}{}",
            super::CSV_ROOT_URL,
            super::AIRPORT_FREQUENCY_CSV
        ))
        .await?;
        let data = result.text().await?;
        let mut reader = csv::ReaderBuilder::new().from_reader(data.as_bytes());

        let con = self.connection.lock().expect(ERROR_SQLITE_ACCESS);

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

    pub async fn load_airport_runways(&self) -> Result<(), Box<dyn Error>> {
        let result = reqwest::get(format!(
            "{}{}",
            super::CSV_ROOT_URL,
            super::AIRPORT_RUNWAY_CSV
        ))
        .await?;
        let data = result.text().await?;
        let mut reader = csv::ReaderBuilder::new().from_reader(data.as_bytes());

        let con = self.connection.lock().expect(ERROR_SQLITE_ACCESS);

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

    pub async fn load_navaids(&self) -> Result<(), Box<dyn Error>> {
        let result = reqwest::get(format!("{}{}", super::CSV_ROOT_URL, super::NAVAID_CSV)).await?;
        let data = result.text().await?;
        let mut reader = csv::ReaderBuilder::new().from_reader(data.as_bytes());

        let con = self.connection.lock().expect(ERROR_SQLITE_ACCESS);

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
                info!("{} navaids loaded", value.unwrap());
            }
            true
        })?;

        Ok(())
    }

    async fn get_list_of_sha(&self) -> Result<HashMap<String, String>, Box<dyn Error>> {
        debug!("Looking for branch sha first");

        let client = reqwest::Client::builder()
            .user_agent(HTTP_USER_AGENT)
            .build()?;

        let result = client.get(super::BRANCH_API).send().await?;

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
            .get(format!("{}{}", super::TREE_API, branch_sha))
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
    fn check_and_store_sha(&self, file: &str, sha: &String) -> Result<bool, Box<dyn Error>> {
        let query = "SELECT count(*) as count FROM data_last_update WHERE file = ? AND sha = ?";

        let con = self.connection.clone();
        let con = con.lock().unwrap();
        let mut s = con.prepare(query)?;
        s.bind((1, file))?;
        s.bind((2, sha.as_str()))?;

        s.next()?;
        let count = s.read::<i64, _>("count")?;
        s.reset()?;

        if count == 0 {
            // sha in DB is different than the one provided, update database
            let query =
                "REPLACE INTO data_last_update (file, sha, date) VALUES (?, ?, unixepoch())";
            let mut s2 = con.prepare(query)?;
            s2.bind((1, file))?;
            s2.bind((2, sha.as_str()))?;
            s2.next()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    pub async fn periodical_update(&self) {
        loop {
            info!("Awake ! reloading data");

            let shas = self.get_list_of_sha().await.unwrap();

            match self
                .check_and_store_sha(super::AIRPORT_CSV, shas.get(super::AIRPORT_CSV).unwrap())
                .unwrap()
            {
                true => {
                    self.load_airports().await.unwrap();
                }
                false => (),
            }

            match self
                .check_and_store_sha(
                    super::AIRPORT_FREQUENCY_CSV,
                    shas.get(super::AIRPORT_FREQUENCY_CSV).unwrap(),
                )
                .unwrap()
            {
                true => {
                    self.load_airport_frequencies().await.unwrap();
                }
                false => (),
            }

            match self
                .check_and_store_sha(
                    super::AIRPORT_RUNWAY_CSV,
                    shas.get(super::AIRPORT_RUNWAY_CSV).unwrap(),
                )
                .unwrap()
            {
                true => {
                    self.load_airport_runways().await.unwrap();
                }
                false => (),
            }

            match self
                .check_and_store_sha(super::NAVAID_CSV, shas.get(super::NAVAID_CSV).unwrap())
                .unwrap()
            {
                true => {
                    self.load_navaids().await.unwrap();
                }
                false => (),
            }

            info!("Database fully reloaded");
            let _delay = sleep(Duration::from_secs(86400)).await;
        }
    }

    pub async fn get_runways_by_icao_code(
        &self,
        icao: String,
    ) -> Result<Vec<Runway>, Box<dyn Error>> {
        let mut runways = vec![];
        {
            let query = "SELECT * FROM airport_runways WHERE airport_icao_code=?";
            let con = self.connection.lock().expect(ERROR_SQLITE_ACCESS);
            let mut runway_statement = con.prepare(query)?;
            runway_statement.bind((1, icao.as_str()))?;

            while let Ok(State::Row) = runway_statement.next() {
                let runway = Runway {
                    id: runway_statement.read::<i64, _>("id")?,
                    airport_id: runway_statement.read::<i64, _>("airport_ref")?,
                    airport_icao_code: runway_statement.read::<String, _>("airport_icao_code")?,
                    length_ft: runway_statement.read::<i64, _>("length_ft")?,
                    width_ft: runway_statement.read::<i64, _>("width_ft")?,
                    surface: runway_statement.read::<i64, _>("surface")?,
                    lighted: runway_statement.read::<i64, _>("lighted")?,
                    closed: runway_statement.read::<i64, _>("closed")?,
                    le_ident: runway_statement.read::<String, _>("le_ident")?,
                    le_location: LocationPoint {
                        r#type: LocationType::Point,
                        coordinates: vec![
                            runway_statement.read::<f64, _>("le_longitude_deg")?,
                            runway_statement.read::<f64, _>("le_latitude_deg")?,
                        ],
                    },
                    le_elevation_ft: runway_statement.read::<i64, _>("le_elevation_ft")?,
                    le_heading_deg_t: runway_statement.read::<i64, _>("le_heading_degT")?,
                    le_displaced_threshold_ft: runway_statement
                        .read::<i64, _>("le_displaced_threshold_ft")?,
                    he_ident: runway_statement.read::<String, _>("he_ident")?,
                    he_location: LocationPoint {
                        r#type: LocationType::Point,
                        coordinates: vec![
                            runway_statement.read::<f64, _>("he_longitude_deg")?,
                            runway_statement.read::<f64, _>("he_latitude_deg")?,
                        ],
                    },
                    he_elevation_ft: runway_statement.read::<i64, _>("he_elevation_ft")?,
                    he_heading_deg_t: runway_statement.read::<i64, _>("he_heading_degT")?,
                    he_displaced_threshold_ft: runway_statement
                        .read::<i64, _>("he_displaced_threshold_ft")?,
                };

                runways.push(runway);
            }
        }
        Ok(runways)
    }

    pub async fn get_frequencies_by_icao_code(
        &self,
        icao: String,
    ) -> Result<Vec<Frequency>, Box<dyn Error>> {
        let mut frequencies = vec![];

        {
            let query = "SELECT * FROM airport_frequencies WHERE airport_icao_code=?";
            let con = self.connection.lock().expect(ERROR_SQLITE_ACCESS);
            let mut freq_statement = con.prepare(query)?;
            freq_statement.bind((1, icao.as_str()))?;

            while let Ok(State::Row) = freq_statement.next() {
                let frequency = Frequency {
                    id: freq_statement.read::<i64, _>("id")?,
                    airport_id: freq_statement.read::<i64, _>("airport_ref")?,
                    airport_icao_code: freq_statement.read::<String, _>("airport_icao_code")?,
                    description: freq_statement.read::<String, _>("description")?,
                    frequency_mhz: freq_statement.read::<f64, _>("frequency_mhz")?,
                    r#type: FrequencyType::from_str(
                        freq_statement.read::<String, _>("type")?.as_str(),
                    )
                    .unwrap(),
                    raw_type: freq_statement.read::<String, _>("type")?,
                };
                frequencies.push(frequency);
            }
        }

        Ok(frequencies)
    }

    pub async fn get_navaids_by_airport_icao_code(
        &self,
        icao: String,
    ) -> Result<Vec<Navaid>, Box<dyn Error>> {
        let mut navaids = vec![];
        let mut ids = vec![];
        {
            let con = self.connection.lock().expect(ERROR_SQLITE_ACCESS);
            let query = "SELECT id FROM navaids WHERE associated_airport=?";
            let mut navaid_statement = con.prepare(query)?;
            navaid_statement.bind((1, icao.as_str()))?;

            while let Ok(State::Row) = navaid_statement.next() {
                ids.push(navaid_statement.read::<i64, _>("id")?);
            }
        }

        for id in ids {
            let navaid = self.get_navaid_by_id(id).await?;
            navaids.push(navaid);
        }
        Ok(navaids)
    }

    pub async fn get_airport_by_icao_code(
        &self,
        icao: String,
    ) -> Result<Option<Airport>, Box<dyn Error>> {
        let query = "SELECT * FROM airports WHERE icao_code=?";
        let icao = icao.to_uppercase();

        let mut airport = Airport::default();
        {
            let con = self.connection.lock().expect(ERROR_SQLITE_ACCESS);
            let mut statement = con.prepare(query)?;
            statement.bind((1, icao.clone().as_str()))?;

            while let Ok(State::Row) = statement.next() {
                airport.id = statement.read::<i64, _>("id")?;
                airport.icao_code = statement.read::<String, _>("icao_code")?;
                airport.r#type =
                    AirportType::from_str(statement.read::<String, _>("type")?.as_str()).unwrap();
                airport.name = statement.read::<String, _>("name")?;
                airport.location = LocationPoint {
                    r#type: LocationType::Point,
                    coordinates: vec![
                        statement.read::<f64, _>("longitude_deg")?,
                        statement.read::<f64, _>("latitude_deg")?,
                    ],
                };
                airport.elevation_ft = statement.read::<i64, _>("elevation_ft")?;
                airport.continent = statement.read::<String, _>("continent")?;
                airport.iso_country = statement.read::<String, _>("iso_country")?;
                airport.iso_region = statement.read::<String, _>("iso_region")?;
                airport.municipality = statement.read::<String, _>("municipality")?;
                airport.scheduled_service = statement.read::<String, _>("scheduled_service")?;
                airport.gps_code = statement.read::<String, _>("gps_code")?;
                airport.iata_code = statement.read::<String, _>("iata_code")?;
                airport.local_code = statement.read::<String, _>("local_code")?;
                airport.home_link = statement.read::<String, _>("home_link")?;
                airport.wikipedia_link = statement.read::<String, _>("wikipedia_link")?;
                airport.keywords = statement.read::<String, _>("keywords")?;
            }
        }
        airport.runways = self.get_runways_by_icao_code(icao.clone()).await?;
        airport.frequencies = self.get_frequencies_by_icao_code(icao.clone()).await?;
        airport.navaids = self.get_navaids_by_airport_icao_code(icao.clone()).await?;
        Ok(Some(airport))
    }

    pub async fn get_navaids_by_icao_code(
        &self,
        icao: String,
    ) -> Result<Vec<Navaid>, Box<dyn Error>> {
        let mut navaids = vec![];
        let query = "SELECT * FROM navaids WHERE icao_code=?";

        let icao = icao.to_uppercase();

        let con = self.connection.lock().expect(ERROR_SQLITE_ACCESS);
        let mut statement = con.prepare(query)?;
        statement.bind((1, icao.as_str()))?;

        while let Ok(State::Row) = statement.next() {
            let navaid = Navaid {
                id: statement.read::<i64, _>("id")?,
                filename: statement.read::<String, _>("filename")?,
                icao_code: statement.read::<String, _>("icao_code")?,
                name: statement.read::<String, _>("elevation_ft")?,
                r#type: NavaidType::from_str(statement.read::<String, _>("type")?.as_str())
                    .unwrap(),
                frequency_khz: statement.read::<i64, _>("frequency_khz")?,
                location: LocationPoint {
                    r#type: LocationType::Point,
                    coordinates: vec![
                        statement.read::<f64, _>("longitude_deg")?,
                        statement.read::<f64, _>("latitude_deg")?,
                    ],
                },
                elevation_ft: statement.read::<i64, _>("elevation_ft")?,
                iso_country: statement.read::<String, _>("iso_country")?,
                dme_frequency_khz: statement.read::<i64, _>("dme_frequency_khz")?,
                dme_channel: statement.read::<String, _>("dme_channel")?,
                dme_location: LocationPoint {
                    r#type: LocationType::Point,
                    coordinates: vec![
                        statement.read::<f64, _>("dme_longitude_deg")?,
                        statement.read::<f64, _>("dme_latitude_deg")?,
                    ],
                },
                dme_elevation_ft: statement.read::<i64, _>("dme_elevation_ft")?,
                slaved_variation_deg: statement.read::<i64, _>("slaved_variation_deg")?,
                magnetic_variation_deg: statement.read::<i64, _>("magnetic_variation_deg")?,
                usage_type: statement.read::<String, _>("usageType")?,
                power: statement.read::<String, _>("power")?,
                associated_airport: statement.read::<String, _>("associated_airport")?,
            };
            navaids.push(navaid);
        }
        Ok(navaids)
    }

    async fn get_navaid_by_id(&self, id: i64) -> Result<Navaid, Box<dyn Error>> {
        let query = "SELECT * FROM navaids WHERE id=?";

        let con = self.connection.lock().expect(ERROR_SQLITE_ACCESS);
        let mut statement = con.prepare(query)?;
        statement.bind((1, id))?;
        statement.next()?;
        let navaid = Navaid {
            id: statement.read::<i64, _>("id")?,
            filename: statement.read::<String, _>("filename")?,
            icao_code: statement.read::<String, _>("icao_code")?,
            name: statement.read::<String, _>("elevation_ft")?,
            r#type: NavaidType::from_str(statement.read::<String, _>("type")?.as_str()).unwrap(),
            frequency_khz: statement.read::<i64, _>("frequency_khz")?,
            location: LocationPoint {
                r#type: LocationType::Point,
                coordinates: vec![
                    statement.read::<f64, _>("longitude_deg")?,
                    statement.read::<f64, _>("latitude_deg")?,
                ],
            },
            elevation_ft: statement.read::<i64, _>("elevation_ft")?,
            iso_country: statement.read::<String, _>("iso_country")?,
            dme_frequency_khz: statement.read::<i64, _>("dme_frequency_khz")?,
            dme_channel: statement.read::<String, _>("dme_channel")?,
            dme_location: LocationPoint {
                r#type: LocationType::Point,
                coordinates: vec![
                    statement.read::<f64, _>("dme_longitude_deg")?,
                    statement.read::<f64, _>("dme_latitude_deg")?,
                ],
            },
            dme_elevation_ft: statement.read::<i64, _>("dme_elevation_ft")?,
            slaved_variation_deg: statement.read::<i64, _>("slaved_variation_deg")?,
            magnetic_variation_deg: statement.read::<i64, _>("magnetic_variation_deg")?,
            usage_type: statement.read::<String, _>("usageType")?,
            power: statement.read::<String, _>("power")?,
            associated_airport: statement.read::<String, _>("associated_airport")?,
        };
        Ok(navaid)
    }

    pub async fn search_navaid(
        &self,
        search: Option<String>,
        page: Option<u64>,
        country: Option<String>,
        navaid_type: Option<String>,
        _latitude: Option<f64>,
        _longitude: Option<f64>,
    ) -> Result<Vec<Navaid>, Box<dyn Error>> {
        let con = self.connection.lock().expect(ERROR_SQLITE_ACCESS);

        // First build the query
        let mut query = "SELECT * FROM navaids".to_owned();
        let mut is_first = true;
        if country.is_some() {
            query.push_str(" WHERE iso_country = ?");
            is_first = false;
        }
        if navaid_type.is_some() {
            if is_first {
                query.push_str(" WHERE");
                is_first = false;
            } else {
                query.push_str(" AND");
            }
            query.push_str(" type = ?");
        }
        if search.is_some() {
            if is_first {
                query.push_str(" WHERE");
                //is_first = false;
            } else {
                query.push_str(" AND");
            }
            query.push_str(" (icao_code LIKE '%' || ? || '%' OR name LIKE '%' || ? || '%' OR associated_airport LIKE '%' || ? || '%')");
        }
        match page {
            Some(page) => query.push_str(format!(" LIMIT {}, 100", page * 100).as_str()),
            None => query.push_str(" LIMIT 100"),
        };

        // Build and fill the statement
        let mut statement = con.prepare(query)?;
        let mut index = 1;
        if country.is_some() {
            let country_param = country.unwrap().to_uppercase();
            statement.bind((index, country_param.as_str()))?;
            index += 1;
        }
        if navaid_type.is_some() {
            let navaid_type_param = navaid_type.unwrap().to_uppercase();
            statement.bind((index, navaid_type_param.as_str()))?;
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
        let mut navaids = vec![];
        while let Ok(State::Row) = statement.next() {
            let navaid = Navaid {
                id: statement.read::<i64, _>("id")?,
                filename: statement.read::<String, _>("filename")?,
                icao_code: statement.read::<String, _>("icao_code")?,
                name: statement.read::<String, _>("elevation_ft")?,
                r#type: NavaidType::from_str(statement.read::<String, _>("type")?.as_str())
                    .unwrap(),
                frequency_khz: statement.read::<i64, _>("frequency_khz")?,
                location: LocationPoint {
                    r#type: LocationType::Point,
                    coordinates: vec![
                        statement.read::<f64, _>("longitude_deg")?,
                        statement.read::<f64, _>("latitude_deg")?,
                    ],
                },
                elevation_ft: statement.read::<i64, _>("elevation_ft")?,
                iso_country: statement.read::<String, _>("iso_country")?,
                dme_frequency_khz: statement.read::<i64, _>("dme_frequency_khz")?,
                dme_channel: statement.read::<String, _>("dme_channel")?,
                dme_location: LocationPoint {
                    r#type: LocationType::Point,
                    coordinates: vec![
                        statement.read::<f64, _>("dme_longitude_deg")?,
                        statement.read::<f64, _>("dme_latitude_deg")?,
                    ],
                },
                dme_elevation_ft: statement.read::<i64, _>("dme_elevation_ft")?,
                slaved_variation_deg: statement.read::<i64, _>("slaved_variation_deg")?,
                magnetic_variation_deg: statement.read::<i64, _>("magnetic_variation_deg")?,
                usage_type: statement.read::<String, _>("usageType")?,
                power: statement.read::<String, _>("power")?,
                associated_airport: statement.read::<String, _>("associated_airport")?,
            };
            navaids.push(navaid);
        }
        Ok(navaids)
    }

    pub async fn search_airport(
        &self,
        search: Option<String>,
        page: Option<u64>,
        country: Option<String>,
        airport_type: Option<String>,
        _latitude: Option<f64>,
        _longitude: Option<f64>,
    ) -> Result<Vec<Airport>, Box<dyn Error>> {
        let codes = {
            let con = self.connection.lock().expect(ERROR_SQLITE_ACCESS);

            // First build the query
            let mut query = "SELECT icao_code FROM airports".to_owned();
            let mut is_first = true;
            if country.is_some() {
                query.push_str(" WHERE iso_country = ?");
                is_first = false;
            }
            if airport_type.is_some() {
                if is_first {
                    query.push_str(" WHERE");
                    is_first = false;
                } else {
                    query.push_str(" AND");
                }
                query.push_str(" type = ?");
            }
            if search.is_some() {
                if is_first {
                    query.push_str(" WHERE");
                    //is_first = false;
                } else {
                    query.push_str(" AND");
                }
                query.push_str(" (icao_code LIKE '%' || ? || '%' OR name LIKE '%' || ? || '%' OR municipality LIKE '%' || ? || '%' OR iata_code LIKE '%' || ? || '%')");
            }
            match page {
                Some(page) => query.push_str(format!(" LIMIT {}, 100", page * 100).as_str()),
                None => query.push_str(" LIMIT 100"),
            };

            // Build and fill the statement
            let mut statement = con.prepare(query)?;
            let mut index = 1;
            if country.is_some() {
                let country_param = country.unwrap().to_uppercase();
                statement.bind((index, country_param.as_str()))?;
                index += 1;
            }
            if airport_type.is_some() {
                let airport_type_param = airport_type.unwrap().to_lowercase();
                statement.bind((index, airport_type_param.as_str()))?;
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

        let mut airports = vec![];
        for code in codes {
            let airport = self.get_airport_by_icao_code(code).await?;
            if airport.is_some() {
                airports.push(airport.unwrap());
            }
        }

        Ok(airports)
    }
}
