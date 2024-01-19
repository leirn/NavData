use actix_web::web;
pub mod mongodb;
pub mod sqlite;
use self::mongodb::MongoDbBackend;
use self::sqlite::SqliteBackend;
use log::error;
use serde::Serialize;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

const BRANCH_API: &str =
    "https://api.github.com/repos/davidmegginson/ourairports-data/branches/main";
const TREE_API: &str = "https://api.github.com/repos/davidmegginson/ourairports-data/git/trees/";

const CSV_ROOT_URL: &str =
    "https://raw.githubusercontent.com/davidmegginson/ourairports-data/main/";
const AIRPORT_CSV: &str = "airports.csv";
const AIRPORT_FREQUENCY_CSV: &str = "airport-frequencies.csv";
const AIRPORT_RUNWAY_CSV: &str = "runways.csv";
const NAVAID_CSV: &str = "navaids.csv";

#[derive(Serialize, Default)]
pub enum AirportType {
    SmallAirport,
    MediumAirport,
    LargeAirport,
    Heliport,
    SeaplaneBase,
    Closed,
    #[default]
    Unknown,
}

impl fmt::Display for AirportType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AirportType::SmallAirport => write!(f, "small_airport"),
            AirportType::MediumAirport => write!(f, "medium_airport"),
            AirportType::LargeAirport => write!(f, "large_airport"),
            AirportType::Heliport => write!(f, "heliport"),
            AirportType::SeaplaneBase => write!(f, "seaplane_base"),
            AirportType::Closed => write!(f, "closed"),
            AirportType::Unknown => write!(f, "unknown"),
        }
    }
}
impl FromStr for AirportType {
    type Err = ();

    fn from_str(input: &str) -> Result<AirportType, Self::Err> {
        match input {
            "small_airport" => Ok(AirportType::SmallAirport),
            "medium_airport" => Ok(AirportType::MediumAirport),
            "large_airport" => Ok(AirportType::LargeAirport),
            "heliport" => Ok(AirportType::Heliport),
            "seaplane_base" => Ok(AirportType::SeaplaneBase),
            "closed" => Ok(AirportType::Closed),
            _ => {
                error!("Unknow airport type {}", input);
                Err(())
            }
        }
    }
}

#[derive(Serialize, Default)]
pub enum FrequencyType {
    Approach,
    Tower,
    Ground,
    Atis,
    Ctaf,
    Arcal,
    Unic,
    Cntr,
    #[default]
    Unknown,
}

impl fmt::Display for FrequencyType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FrequencyType::Approach => write!(f, "APP"),
            FrequencyType::Tower => write!(f, "TWR"),
            FrequencyType::Ground => write!(f, "GND"),
            FrequencyType::Atis => write!(f, "ATIS"),
            FrequencyType::Ctaf => write!(f, "CTAF"),
            FrequencyType::Arcal => write!(f, "ARCAL"),
            FrequencyType::Unic => write!(f, "UNIC"),
            FrequencyType::Cntr => write!(f, "CNTR"),
            FrequencyType::Unknown => write!(f, "unknown"),
        }
    }
}
impl FromStr for FrequencyType {
    type Err = ();

    fn from_str(input: &str) -> Result<FrequencyType, Self::Err> {
        match input {
            "APP" => Ok(FrequencyType::Approach),
            "TWR" => Ok(FrequencyType::Tower),
            "GND" => Ok(FrequencyType::Ground),
            "ATIS" => Ok(FrequencyType::Atis),
            "CTAF" => Ok(FrequencyType::Ctaf),
            "ARCAL" => Ok(FrequencyType::Arcal),
            "UNIC" => Ok(FrequencyType::Unic),
            "CNTR" => Ok(FrequencyType::Cntr),
            _ => {
                error!("Unknow frequency type {}", input);
                Err(())
            }
        }
    }
}

#[derive(Serialize, Default)]
pub enum NavaidType {
    Vor,
    VorDme,
    Dme,
    Adf,
    VorTac,
    Tacan,
    Ndb,
    NdbDme,
    #[default]
    Unknown,
}

impl fmt::Display for NavaidType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NavaidType::Vor => write!(f, "VOR"),
            NavaidType::VorDme => write!(f, "VOR-DME"),
            NavaidType::Dme => write!(f, "DME"),
            NavaidType::VorTac => write!(f, "VORTAC"),
            NavaidType::Tacan => write!(f, "TACAN"),
            NavaidType::Adf => write!(f, "ADF"),
            NavaidType::Ndb => write!(f, "NDB"),
            NavaidType::NdbDme => write!(f, "NDB-DME"),
            NavaidType::Unknown => write!(f, "unknown"),
        }
    }
}
impl FromStr for NavaidType {
    type Err = ();

    fn from_str(input: &str) -> Result<NavaidType, Self::Err> {
        match input {
            "VOR" => Ok(NavaidType::Vor),
            "VOR-DME" => Ok(NavaidType::VorDme),
            "DME" => Ok(NavaidType::Dme),
            "VORTAC" => Ok(NavaidType::VorTac),
            "ADF" => Ok(NavaidType::Adf),
            "NDB" => Ok(NavaidType::Ndb),
            "NDB-DME" => Ok(NavaidType::NdbDme),
            "TACAN" => Ok(NavaidType::Tacan),
            _ => {
                error!("Unknow navaid type {}", input);
                Err(())
            }
        }
    }
}

#[derive(Serialize, Default)]
pub struct LocationPoint {
    r#type: LocationType,
    coordinates: Vec<f64>,
}

#[derive(Serialize, Default)]
pub enum LocationType {
    #[default]
    Point,
}

impl fmt::Display for LocationType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LocationType::Point => write!(f, "Point"),
        }
    }
}
impl FromStr for LocationType {
    type Err = ();

    fn from_str(input: &str) -> Result<LocationType, Self::Err> {
        match input {
            "Point" => Ok(LocationType::Point),
            _ => Err(()),
        }
    }
}

#[derive(Serialize, Default)]
pub struct Airport {
    pub id: i64,
    pub icao_code: String,
    pub r#type: AirportType,
    pub name: String,
    pub location: LocationPoint,
    pub elevation_ft: i64,
    pub continent: String,
    pub iso_country: String,
    pub iso_region: String,
    pub municipality: String,
    pub scheduled_service: String,
    pub gps_code: String,
    pub iata_code: String,
    pub local_code: String,
    pub home_link: String,
    pub wikipedia_link: String,
    pub keywords: String,
    pub runways: Vec<Runway>,
    pub frequencies: Vec<Frequency>,
    pub navaids: Vec<Navaid>,
}

#[derive(Serialize, Default)]
pub struct Runway {
    pub id: i64,
    pub airport_id: i64,
    pub airport_icao_code: String,
    pub length_ft: i64,
    pub width_ft: i64,
    pub surface: i64,
    pub lighted: i64,
    pub closed: i64,
    pub le_ident: String,
    pub le_location: LocationPoint,
    pub le_elevation_ft: i64,
    pub le_heading_deg_t: i64,
    pub le_displaced_threshold_ft: i64,
    pub he_ident: String,
    pub he_location: LocationPoint,
    pub he_elevation_ft: i64,
    pub he_heading_deg_t: i64,
    pub he_displaced_threshold_ft: i64,
}
#[derive(Serialize, Default)]
pub struct Frequency {
    pub id: i64,
    pub airport_id: i64,
    pub airport_icao_code: String,
    pub r#type: FrequencyType,
    pub description: String,
    pub frequency_mhz: f64,
}
#[derive(Serialize, Default)]
pub struct Navaid {
    pub id: i64,
    pub filename: String,
    pub icao_code: String,
    pub name: String,
    pub r#type: NavaidType,
    pub frequency_khz: i64,
    pub location: LocationPoint,
    pub elevation_ft: i64,
    pub iso_country: String,
    pub dme_frequency_khz: i64,
    pub dme_channel: String,
    pub dme_location: LocationPoint,
    pub dme_elevation_ft: i64,
    pub slaved_variation_deg: i64,
    pub magnetic_variation_deg: i64,
    pub usage_type: String,
    pub power: String,
    pub associated_airport: String,
}

pub struct AppState {
    pub database: DatabaseBackend,
}

pub async fn periodical_update(app_state: web::Data<AppState>) {
    let state = app_state.clone();
    state.database.periodical_update().await
}

#[derive(Clone, Copy)]
pub enum BackendType {
    SQLITE,
    MONGODB,
}

pub struct DatabaseBackend {
    sqlite: Option<SqliteBackend>,
    mongo: Option<MongoDbBackend>,
    active_backend: BackendType,
}
impl DatabaseBackend {
    pub async fn new(backend_type: BackendType, path: String) -> DatabaseBackend {
        let mut backend = DatabaseBackend {
            sqlite: None,
            mongo: None,
            active_backend: backend_type,
        };
        match backend_type {
            BackendType::MONGODB => {
                let database = MongoDbBackend::new(path.as_str()).await;
                backend.mongo = Some(database)
            }
            BackendType::SQLITE => {
                let database = SqliteBackend::new(path);
                backend.sqlite = Some(database)
            }
        }
        backend
    }

    pub async fn periodical_update(&self) {
        match self.active_backend {
            BackendType::MONGODB => self.mongo.as_ref().unwrap().periodical_update().await,
            BackendType::SQLITE => self.sqlite.as_ref().unwrap().periodical_update().await,
        }
    }
    pub async fn get_airport_by_icao_code(&self, icao: String) -> Result<Airport, Box<dyn Error>> {
        match self.active_backend {
            BackendType::MONGODB => {
                self.mongo
                    .as_ref()
                    .unwrap()
                    .get_airport_by_icao_code(icao)
                    .await
            }
            BackendType::SQLITE => {
                self.sqlite
                    .as_ref()
                    .unwrap()
                    .get_airport_by_icao_code(icao)
                    .await
            }
        }
    }

    pub async fn get_navaids_by_icao_code(
        self: &Self,
        icao: String,
    ) -> Result<Vec<Navaid>, Box<dyn Error>> {
        match self.active_backend {
            BackendType::MONGODB => {
                self.mongo
                    .as_ref()
                    .unwrap()
                    .get_navaids_by_icao_code(icao)
                    .await
            }
            BackendType::SQLITE => {
                self.sqlite
                    .as_ref()
                    .unwrap()
                    .get_navaids_by_icao_code(icao)
                    .await
            }
        }
    }

    pub async fn get_navaid_by_id(self: &Self, id: i64) -> Result<Navaid, Box<dyn Error>> {
        match self.active_backend {
            BackendType::MONGODB => self.mongo.as_ref().unwrap().get_navaid_by_id(id).await,
            BackendType::SQLITE => self.sqlite.as_ref().unwrap().get_navaid_by_id(id).await,
        }
    }
    pub async fn search_navaid(
        self: &Self,
        search: Option<String>,
        page: Option<u32>,
        country: Option<String>,
        navaid_type: Option<String>,
    ) -> Result<Vec<Navaid>, Box<dyn Error>> {
        match self.active_backend {
            BackendType::MONGODB => {
                self.mongo
                    .as_ref()
                    .unwrap()
                    .search_navaid(search, page, country, navaid_type)
                    .await
            }
            BackendType::SQLITE => {
                self.sqlite
                    .as_ref()
                    .unwrap()
                    .search_navaid(search, page, country, navaid_type)
                    .await
            }
        }
    }
    pub async fn search_airport(
        self: &Self,
        search: Option<String>,
        page: Option<u32>,
        country: Option<String>,
        airport_type: Option<String>,
    ) -> Result<Vec<Airport>, Box<dyn Error>> {
        match self.active_backend {
            BackendType::MONGODB => {
                self.mongo
                    .as_ref()
                    .unwrap()
                    .search_airport(search, page, country, airport_type)
                    .await
            }
            BackendType::SQLITE => {
                self.sqlite
                    .as_ref()
                    .unwrap()
                    .search_airport(search, page, country, airport_type)
                    .await
            }
        }
    }
}
