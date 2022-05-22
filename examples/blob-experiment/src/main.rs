use anyhow::Result;

use event_handler::*;
use resources::{Bucket, SqlDatabase, Events};

wit_bindgen_rust::import!("../../wit/experimental/resources.wit");
wit_bindgen_rust::export!("../../wit/experimental/event-handler.wit");

fn main() -> Result<()> {
    let bucket = Bucket::get()?;
    let sql_database = SqlDatabase::new()?;

    let events = Events::get()?;
    events
        .listen(&bucket.on("my-key")?)?
        .listen(&sql_database.on("my-table")?)?
        .listen(&bucket.on("my-key2")?)?
        .exec()
        .map_err(|e| e.into())
}

impl From<resources::Error> for anyhow::Error {
    fn from(e: resources::Error) -> Self {
        anyhow::anyhow!("blob error: {:?}", e)
    }
}

pub struct EventHandler {}

impl event_handler::EventHandler for EventHandler {
    fn handle_event(ev: event_handler::Event,) -> Result<Option<event_handler::Event>,Error> {
        println!("{:?}", ev.source);
        println!("{:?}", ev.event_type);
        Ok(Some(ev))
    }
}