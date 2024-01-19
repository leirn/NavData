use bson::doc;
use log::info;
use mongodb::{options::ClientOptions, Client, Collection};
use std::error::Error;

use super::{sqlite::SqliteBackend, Airport, Navaid};

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

        // check if collections exists, create if non
        let coll_list: Vec<String> = client
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
            client
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
            client
                .database(DATABASE_NAME)
                .create_collection(NAVAIDS_COLLECTION, None)
                .await
                .unwrap();
        }

        // let my_coll: Collection<Book> = client.database("db").collection("books");
        // let doc = Book {
        //     _id: 8,
        //     title: "Atonement".to_string(),
        //     author: "Ian McEwan".to_string(),
        // };
        // let insert_one_result = my_coll.insert_one(doc, None).await?;
        // println!(
        //     "Inserted document with _id: {}",
        //     insert_one_result.inserted_id
        // );
        let backend = MongoDbBackend {
            client: client.clone(),
        };
        backend.load_database().await;
        backend
    }

    async fn load_database(&self) {
        let sqlite_be = SqliteBackend::new(":memory:".to_string());

        let airports_collection: Collection<Airport> = self
            .client
            .database(DATABASE_NAME)
            .collection(AIRPORTS_COLLECTION);
        loop {
            let airports = sqlite_be
                .search_airport(None, None, None, None)
                .await
                .unwrap();

            // airports_collection.insert_many(airports, options);

            // if airports.len() == 0 {
            //     break;
            // }
        }
    }

    pub async fn periodical_update(&self) {}
    pub async fn get_airport_by_icao_code(&self, icao: String) -> Result<Airport, Box<dyn Error>> {
        Ok(Airport::default())
    }
    pub async fn get_navaids_by_icao_code(
        &self,
        icao: String,
    ) -> Result<Vec<Navaid>, Box<dyn Error>> {
        Ok(vec![])
    }
    pub async fn get_navaid_by_id(&self, id: i64) -> Result<Navaid, Box<dyn Error>> {
        Ok(Navaid::default())
    }
    pub async fn search_navaid(
        &self,
        search: Option<String>,
        page: Option<u32>,
        country: Option<String>,
        navaid_type: Option<String>,
    ) -> Result<Vec<Navaid>, Box<dyn Error>> {
        Ok(vec![Navaid::default()])
    }
    pub async fn search_airport(
        &self,
        search: Option<String>,
        page: Option<u32>,
        country: Option<String>,
        airport_type: Option<String>,
    ) -> Result<Vec<Airport>, Box<dyn Error>> {
        Ok(vec![Airport::default()])
    }
}
