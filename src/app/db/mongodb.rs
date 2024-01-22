use super::{sqlite::SqliteBackend, Airport, Navaid};
use bson::doc;
use futures::stream::TryStreamExt;
use log::info;
use mongodb::{
    options::{ClientOptions, FindOptions, IndexOptions, ReplaceOptions, Sphere2DIndexVersion},
    Client, Collection, IndexModel,
};
use std::error::Error;
use tokio::time::{sleep, Duration};

const APP_NAME: &str = "navdata";
const DATABASE_NAME: &str = "navdata";
const AIRPORTS_COLLECTION: &str = "airports";
const NAVAIDS_COLLECTION: &str = "navaids";

pub struct MongoDbBackend {
    client: Client,
}

impl MongoDbBackend {
    pub async fn new(database_adress: &str) -> MongoDbBackend {
        let mut client_options = ClientOptions::parse(database_adress).await.unwrap();
        client_options.app_name = Some(APP_NAME.to_string());

        // Get a handle to the deployment.
        let client = Client::with_options(client_options).unwrap();

        // List the names of the databases in that deployment.
        info!("Check if database {}  exists", DATABASE_NAME);
        let mut database_exists = false;
        for db_name in client.list_database_names(None, None).await.unwrap() {
            println!("{}", db_name);
            if db_name == DATABASE_NAME {
                database_exists = true;
                break;
            }
        }
        if !database_exists {
            panic!("{} database does not exists", DATABASE_NAME);
        }
        info!("Found database {}. Continue", DATABASE_NAME);

        let backend = MongoDbBackend {
            client: client.clone(),
        };
        backend.create_collections().await;
        backend.create_indexes().await;
        backend
    }

    async fn create_collections(&self) {
        // check if collections exists, create if non
        let coll_list: Vec<String> = self
            .client
            .database(DATABASE_NAME)
            .list_collection_names(doc! {})
            .await
            .unwrap();
        println!("{:?}", coll_list);
        if !coll_list.contains(&AIRPORTS_COLLECTION.to_string()) {
            info!(
                "Collection {} does not exists. Creating...",
                AIRPORTS_COLLECTION
            );
            self.client
                .database(DATABASE_NAME)
                .create_collection(AIRPORTS_COLLECTION, None)
                .await
                .unwrap();
        }
        if !coll_list.contains(&NAVAIDS_COLLECTION.to_string()) {
            info!(
                "Collection {} does not exists. Creating...",
                NAVAIDS_COLLECTION
            );
            self.client
                .database(DATABASE_NAME)
                .create_collection(NAVAIDS_COLLECTION, None)
                .await
                .unwrap();
        }
    }

    async fn create_indexes(&self) {
        // https://www.mongodb.com/docs/drivers/rust/current/fundamentals/indexes/

        let airports_collection: Collection<Airport> = self
            .client
            .database(DATABASE_NAME)
            .collection(AIRPORTS_COLLECTION);
        let navaids_collection: Collection<Navaid> = self
            .client
            .database(DATABASE_NAME)
            .collection(NAVAIDS_COLLECTION);

        // id must be unique on all collections for updates
        let option = IndexOptions::builder().unique(true).build();
        let index_model = IndexModel::builder()
            .keys(doc! { "id": 1 })
            .options(option)
            .build();
        let index = airports_collection
            .create_index(index_model.clone(), None)
            .await
            .unwrap();
        info!("Index {} created for airports collection", index.index_name);
        let index = navaids_collection
            .create_index(index_model, None)
            .await
            .unwrap();
        info!("Index {} created for navaid collection", index.index_name);

        // icao_code are unique for airports must be unique on all collections for updates
        let option = IndexOptions::builder().unique(true).build();
        let index_model = IndexModel::builder()
            .keys(doc! { "iaco_code": "text" })
            .options(option)
            .build();
        let index = airports_collection
            .create_index(index_model.clone(), None)
            .await
            .unwrap();
        info!("Index {} created for airports collection", index.index_name);

        // name, municipality, iata_code, iso_country and type are mandatory to speed_up searchs on airports
        let index_model = IndexModel::builder()
            .keys(
                doc! { "name": 1, "municipality": 1, "iata_code": 1, "iso_country": 1, "type": 1  },
            )
            .build();
        match airports_collection
            .create_index(index_model.clone(), None)
            .await
        {
            Ok(index) => info!("Index {} created for airports collection", index.index_name),
            Err(err) => info!("Index not created, may alreay exists : {}", err),
        }

        // name, filename, associated_airport, iso_country and type are mandatory to speed_up searchs on navaids
        let index_model = IndexModel::builder()
            .keys(
                doc! { "name": 1,"filename": 1,"associated_airport": 1,"type": 1,"iso_country": 1 },
            )
            .build();
        match navaids_collection
            .create_index(index_model.clone(), None)
            .await
        {
            Ok(index) => info!("Index {} created for navaids collection", index.index_name),
            Err(err) => info!("Index not created, may alreay exists : {}", err),
        }

        // locations are geo-indexed
        let option = IndexOptions::builder()
            .sphere_2d_index_version(Sphere2DIndexVersion::V3)
            .build();
        let index_model = IndexModel::builder()
            .keys(doc! { "location": "2dsphere" })
            .options(option)
            .build();
        let index = airports_collection
            .create_index(index_model.clone(), None)
            .await
            .unwrap();
        info!("Index {} created for airports collection", index.index_name);
        let index = navaids_collection
            .create_index(index_model, None)
            .await
            .unwrap();
        info!("Index {} created for navaid collection", index.index_name);
    }

    async fn load_database(&self) {
        // Loading data to sqlite temporarly
        let sqlite_be = SqliteBackend::new(":memory:".to_string());
        sqlite_be.load_airports().await.unwrap();
        sqlite_be.load_airport_frequencies().await.unwrap();
        sqlite_be.load_airport_runways().await.unwrap();
        sqlite_be.load_navaids().await.unwrap();

        // Copying airports from sqlite to mongodb
        let airports_collection: Collection<Airport> = self
            .client
            .database(DATABASE_NAME)
            .collection(AIRPORTS_COLLECTION);
        info!("Start adding airports");
        let mut airport_count = 0;
        let mut page = 0;
        loop {
            let airports = sqlite_be
                .search_airport(None, Some(page), None, None, None, None)
                .await
                .unwrap();

            if airports.len() == 0 {
                break;
            }

            // let result = airports_collection
            //     .insert_many(airports, None)
            //     .await
            //     .unwrap();

            for airport in airports {
                if airport.icao_code.len() < 4 {
                    continue;
                }
                let option = ReplaceOptions::builder().upsert(true).build();
                airports_collection
                    .replace_one(doc! { "id": airport.id }, airport, Some(option))
                    .await
                    .unwrap();
                airport_count += 1;
            }
            // airport_count += result.inserted_ids.len();
            page += 1
        }
        info!("{} airports added to MongoDB", airport_count);

        // Copying navids from sqlite to mongodb
        let navaids_collection: Collection<Navaid> = self
            .client
            .database(DATABASE_NAME)
            .collection(NAVAIDS_COLLECTION);
        info!("Start adding navaid");
        let mut navaid_count = 0;
        let mut page = 0;
        loop {
            let navaids = sqlite_be
                .search_navaid(None, Some(page), None, None, None, None)
                .await
                .unwrap();

            if navaids.len() == 0 {
                break;
            }
            let result = navaids_collection.insert_many(navaids, None).await.unwrap();
            navaid_count += result.inserted_ids.len();
            page += 1;
        }
        info!("{} navaids added to MongoDB", navaid_count);
    }

    pub async fn periodical_update(&self) {
        loop {
            info!("Awake ! reloading data");
            self.load_database().await;
            info!("Database fully reloaded");
            let _delay = sleep(Duration::from_secs(86400)).await;
        }
    }

    pub async fn get_airport_by_icao_code(
        &self,
        icao: String,
    ) -> Result<Option<Airport>, Box<dyn Error>> {
        let coll: Collection<Airport> = self
            .client
            .database(DATABASE_NAME)
            .collection(AIRPORTS_COLLECTION);
        let result = coll.find_one(doc! {"icao_code":icao}, None).await?;
        Ok(result)
    }
    pub async fn get_navaids_by_icao_code(
        &self,
        icao: String,
    ) -> Result<Vec<Navaid>, Box<dyn Error>> {
        let coll: Collection<Navaid> = self
            .client
            .database(DATABASE_NAME)
            .collection(NAVAIDS_COLLECTION);
        let mut result = coll.find(doc! {"icao_code":icao}, None).await?;
        let mut navaids = vec![];
        while let Some(navaid) = result.try_next().await? {
            navaids.push(navaid);
        }
        Ok(navaids)
    }
    pub async fn search_navaid(
        &self,
        search: Option<String>,
        page: Option<u64>,
        country: Option<String>,
        navaid_type: Option<String>,
        latitude: Option<f64>,
        longitude: Option<f64>,
    ) -> Result<Vec<Navaid>, Box<dyn Error>> {
        let coll: Collection<Navaid> = self
            .client
            .database(DATABASE_NAME)
            .collection(NAVAIDS_COLLECTION);

        let mut ands = vec![];

        if search.is_some() {
            let search = search.unwrap();
            let search_filter = doc! {"$or": [
            {"icao_code":search.clone()},
            {"name":search.clone()},
            {"associated_airport":search.clone()}
            ]};
            ands.push(search_filter);
        }

        if latitude.is_some() && longitude.is_some() {
            let geo_filter = doc! {"location":{
              "$nearSphere": {
                 "$geometry": {
                    "type" : "Point",
                    "coordinates" : [ longitude.unwrap(), latitude.unwrap() ]
                 },
                 "$minDistance": 0,
                 "$maxDistance": 5000000
              }
            }};
            ands.push(geo_filter);
        }

        if country.is_some() {
            let country_filter = doc! {"iso_country": country.unwrap()};
            ands.push(country_filter);
        }

        if navaid_type.is_some() {
            let type_filter = doc! {"type": navaid_type.unwrap()};
            ands.push(type_filter);
        }

        let filter = doc! {"$and":ands};

        let page = page.unwrap_or(0);
        let options = FindOptions::builder().skip(page * 100).limit(100).build();

        let mut result = coll.find(filter, options).await?;
        let mut navaids = vec![];
        while let Some(navaid) = result.try_next().await? {
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
        latitude: Option<f64>,
        longitude: Option<f64>,
    ) -> Result<Vec<Airport>, Box<dyn Error>> {
        let coll: Collection<Airport> = self
            .client
            .database(DATABASE_NAME)
            .collection(AIRPORTS_COLLECTION);

        let mut ands = vec![];

        if search.is_some() {
            let search = search.unwrap();
            let search_filter = doc! {"$or": [
            {"icao_code":search.clone()},
            {"name":search.clone()},
            {"municipality":search.clone()},
            {"iata_code":search.clone()}
            ]};
            ands.push(search_filter);
        }

        if latitude.is_some() && longitude.is_some() {
            let geo_filter = doc! {"location":{
              "$nearSphere": {
                 "$geometry": {
                    "type" : "Point",
                    "coordinates" : [ longitude.unwrap(), latitude.unwrap() ]
                 },
                 "$minDistance": 0,
                 "$maxDistance": 5000000
              }
            }};
            ands.push(geo_filter);
        }

        if country.is_some() {
            let country_filter = doc! {"iso_country": country.unwrap()};
            ands.push(country_filter);
        }

        if airport_type.is_some() {
            let type_filter = doc! {"type": airport_type.unwrap()};
            ands.push(type_filter);
        }

        let filter = doc! {"$and":ands};

        let page = page.unwrap_or(0);
        let options = FindOptions::builder().skip(page * 100).limit(100).build();

        let mut result = coll.find(filter, options).await?;
        let mut airports = vec![];
        while let Some(airport) = result.try_next().await? {
            airports.push(airport);
        }
        Ok(airports)
    }
}
