use crate::app::db::DatabaseBackend;
use crate::app::messages::*;
use crate::app::messages::{CSV_FORMAT_ERROR, ERROR_SQLITE_ACCESS, HTTP_USER_AGENT};
use ::sqlite::Connection;
use async_trait::async_trait;
use log::{debug, info};
use serde_json::{json, Value};
use sqlite::State;
use std::env;
use std::error::Error;
use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};
use tokio::time::{sleep, Duration};

pub struct SqliteBackend {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteBackend {
    pub fn new() -> SqliteBackend {
        let path = env::var(PARAM_DATABASE_PATH).unwrap_or(String::from(DEFAULT_DATABASE));
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

    async fn load_airports(&self) -> Result<(), Box<dyn Error>> {
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

    async fn load_airport_frequencies(&self) -> Result<(), Box<dyn Error>> {
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

    async fn load_airport_runways(&self) -> Result<(), Box<dyn Error>> {
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

    async fn load_navaids(&self) -> Result<(), Box<dyn Error>> {
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
                info!("{} airport runways loaded", value.unwrap());
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
}

#[async_trait]
impl DatabaseBackend for SqliteBackend {
    async fn periodical_update(&self) {
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

    async fn get_airport_by_icao_code(&self, icao: String) -> Result<Value, Box<dyn Error>> {
        let query = "SELECT * FROM airports WHERE icao_code=?";
        let icao = icao.to_uppercase();

        let mut data = json!({});
        {
            let con = self.connection.lock().expect(ERROR_SQLITE_ACCESS);
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
                navaids.as_array_mut().unwrap().push(navaid);
            }
        }
        data.as_object_mut()
            .unwrap()
            .insert(String::from("navaids"), navaids);
        Ok(data)
    }

    async fn get_navaid_by_icao_code(&self, icao: String) -> Result<Value, Box<dyn Error>> {
        let query = "SELECT * FROM navaids WHERE icao_code=?";

        let icao = icao.to_uppercase();

        let mut data = json!([]);

        let con = self.connection.lock().expect(ERROR_SQLITE_ACCESS);
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

    async fn get_navaid_by_id(&self, id: i64) -> Result<Value, Box<dyn Error>> {
        let query = "SELECT * FROM navaids WHERE id=?";

        let con = self.connection.lock().expect(ERROR_SQLITE_ACCESS);
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

    async fn search_navaid(
        &self,
        search: Option<String>,
        page: Option<u32>,
        country: Option<String>,
        navaid_type: Option<String>,
    ) -> Result<Value, Box<dyn Error>> {
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

    async fn search_airport(
        &self,
        search: Option<String>,
        page: Option<u32>,
        country: Option<String>,
        airport_type: Option<String>,
    ) -> Result<Value, Box<dyn Error>> {
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

        let mut data = json!([]);
        for code in codes {
            let airport_data = self.get_airport_by_icao_code(code).await?;
            data.as_array_mut().unwrap().push(airport_data);
        }

        Ok(data)
    }
}
