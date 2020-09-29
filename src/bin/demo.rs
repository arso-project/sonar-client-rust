#[allow(dead_code)]
// use anyhow::Result;
use async_std::task;
use eyre::Result;
use futures::future;
use futures::stream::StreamExt;
use log::*;
use serde_json::Value;
use sonar_client_rust::schema;
use sonar_client_rust::{Client, Collection};
use thiserror::Error;

#[async_std::main]
async fn main() -> Result<()> {
    env_logger::init();
    let name = std::env::args().nth(1).expect("pass subscription name arg");

    let client = Client::default();
    let collection = client.collection("default");

    // Subscribe to the event stream and log all events.
    // TODO: surf_sse::EventSource is not Send, so we have to use
    // spawn_local.
    let mut tasks = vec![];
    let task = task::spawn_local(log_events(collection.clone()));
    tasks.push(task);
    let task = task::spawn_local(log_subscription(collection.clone(), name));
    tasks.push(task);
    let task = task::spawn_local(log_query(collection));
    tasks.push(task);

    let res = future::join_all(tasks).await;
    eprintln!("res {:?}", res);

    Ok(())
}

async fn log_subscription(collection: Collection, name: impl ToString) -> Result<()> {
    // Subscribe to the collection under a name.
    let mut subscription = collection.subscribe(name.to_string());
    while let Some(batch) = subscription.next().await {
        // info!("got batch: {:?}", batch);
        let batch = batch.unwrap();
        for record in batch.messages {
            log_record(record)?;
        }
    }
    Ok(())
}

async fn log_query(collection: Collection) -> Result<()> {
    let args = serde_json::json!("hei");
    let query = "search";
    let res = collection.query(query, args).await.unwrap();
    eprintln!("{} results", res.len());
    for record in res {
        log_record(record)?;
    }
    Ok(())
}

async fn log_events(collection: Collection) -> Result<()> {
    // TODO: Handle error.
    let mut events = collection.events().unwrap();
    eprintln!("now log events");
    while let Some(event) = events.next().await {
        eprintln!("[{}] Event {:?}", collection.name(), event);
    }
    Ok(())
}

#[derive(Error, Debug)]
enum ParseError {
    #[error("Not an object")]
    NotAnObject,
}

fn log_record(record: schema::Record) -> std::result::Result<(), ParseError> {
    let typ = record.r#type;
    let value: Value = serde_json::from_str(record.value.unwrap().get()).unwrap();
    let value = if let Value::Object(value) = value {
        Ok(value)
    } else {
        Err(ParseError::NotAnObject)
    }?;

    eprintln!(
        "\n lseq: {} id: {} type: {}",
        record.lseq.unwrap(),
        record.id,
        typ
    );
    for (name, value) in value.iter() {
        let field_address = format!("{}#{}", typ, name);
        eprintln!("   {}: {:?}", field_address, value);
    }
    Ok(())
}

// fn parallel(collection: Collection) -> Result<()> {
// let mut tasks = vec![];
// let instant = std::time::Instant::now();
// let par: u32 = 1;
// let seq: u32 = 1;
// for _i in 0..par {
//     let collection = collection.clone();
//     let name = name.clone();
//     let seq = seq.clone();
//     let task = task::spawn(async move {
//         for _j in 0..seq {
//             let instant = std::time::Instant::now();
//             if let Err(err) = subscribe(&collection, name.clone()).await {
//                 eprintln!("ERROR: {:?}", err)
//             }
//         }
//     });
//     tasks.push(task);
// }
// eprintln!("tasks created {:?}", instant.elapsed());
// futures::future::join_all(tasks).await;
// eprintln!("tasks done {:?}", instant.elapsed());
// eprintln!("per task {:?}", instant.elapsed() / (par * seq));
// }
