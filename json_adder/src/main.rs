#[allow(unused_imports)]
#[allow(dead_code)]
use bson::Document;
use chrono::{NaiveDateTime, Utc};
use core::fmt;
use mongodb::{
    bson,
    options::{ClientOptions, ServerApi, ServerApiVersion},
    Client,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;

#[tokio::main]

async fn main() -> mongodb::error::Result<()> {
    let uri: &str = "mongodb://ADMIN:PASSWORD@localhost:27017";
    let mut client_options = ClientOptions::parse_async(uri).await?;

    // Set the server_api field of the client_options object to Stable API version 1
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);

    // Create a new client and connect to the server
    let client: Client = Client::with_options(client_options)?;

    // Connect to the database and the snapshot collection document
    let database = client.database("communities");
    let snapshot_collection = database.collection("hourlySnapshot");

    // File path for sample json file, change this later
    let file_path: &str =
        "../../api.freifunk.net/data/history/20240129-10.01.02-ffSummarizedDir.json";

    // Convert JSON to string, then to value, then to bson
    let contents: String = fs::read_to_string(file_path).expect("couldn't read file");
    let value: Value = serde_json::from_str(&contents).expect("couldn't parse json");

    #[derive(Serialize, Deserialize, Debug)]
    struct Community {
        label: String,
        timestamp: bson::DateTime,
        content: Value,
    }

    // Construct bson datetime function
    fn mtime_to_bson (mtime: &str) -> bson::DateTime {
        // mtime is surrounded by quotes, and when passed into parse_from_str, it is
        // cut down to the format described in fmt
        let fmt = "%Y-%m-%d %H:%M:%S";
        let chrono_dt: chrono::DateTime<Utc> = NaiveDateTime::parse_from_str(&mtime[1..20], fmt)
            .expect("failed to parse naive datetime from json value")
            .and_utc();
        // Convert to bson datetime, copied
        // from https://docs.rs/bson/latest/bson/struct.DateTime.html
        let bson_dt: bson::DateTime = chrono_dt.into();
        bson::DateTime::from_chrono(chrono_dt);
        bson_dt
    }

    for (community_label, community_info) in value.as_object().unwrap() {
       
        let mtime = &community_info["mtime"].to_string();
        let bson_time = mtime_to_bson(mtime);
        println!("{:?}", bson_time);

        let community = Community {
            label: community_label.to_string(),
            timestamp: bson_time,
            content: community_info.clone()
        };


        // let bson_doc: Document = bson::to_bson(&value)
    //     .expect("couldn't convert value to bson")
    //     .as_document()
    //     .unwrap()
    //     .clone();

    // Insert this the bson into the collection
    let result = snapshot_collection.insert_one(community, None).await?;

    println!("Inserted a document with _id: {}", result.inserted_id);
    
    }



    Ok(())
}
