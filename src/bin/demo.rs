// use anyhow::Result;
use async_std::task;
use eyre::Result;
use futures::channel::{mpsc, oneshot};
use futures::future::FutureExt;
use futures::stream::{Stream, StreamExt, TryStreamExt};
use log::*;
use serde_json::Value;
use sonar_client_rust::schema;
use sonar_client_rust::{Client, Collection};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use thiserror::Error;

#[async_std::main]
async fn main() -> Result<()> {
    env_logger::init();
    let name = std::env::args().nth(1).expect("pass subscription name arg");

    let client = Client::default();
    let collection = Collection::new(client, "default");

    // Subscribe to the event stream and log all events.
    // TODO: surf_sse::EventSource is not Send, so we have to use
    // spawn_local.
    // let log_events = task::spawn_local(log_events(collection.clone()));
    let log_subscription = task::spawn_local(log_subscription(collection, name));

    // log_events.await?;
    log_subscription.await?;

    Ok(())
}

async fn log_subscription(collection: Collection, name: impl ToString) -> Result<()> {
    // Subscribe to the collection under a name.
    let mut subscription = collection.subscribe(name.to_string());
    while let Some(batch) = subscription.next().await {
        info!("got batch: {:?}", batch);
        let batch = batch.unwrap();
        on_batch(batch).unwrap();
    }
    Ok(())
}

async fn log_events(collection: Collection) -> Result<()> {
    // TODO: Handle error.
    let mut events = collection.events().unwrap();
    while let Some(event) = events.next().await {
        info!("[{}] Event {:?}", collection.name(), event);
    }
    Ok(())
}

fn on_batch(batch: schema::PullResponse) -> Result<()> {
    for record in batch.messages {
        parse_value(record)?;
    }
    Ok(())
}

#[derive(Error, Debug)]
enum ParseError {
    #[error("Not an object")]
    NotAnObject,
}

fn parse_value(record: schema::Record) -> std::result::Result<(), ParseError> {
    let typ = record.r#type;
    let value: Value = serde_json::from_str(record.value.unwrap().get()).unwrap();
    let value = if let Value::Object(value) = value {
        Ok(value)
    } else {
        Err(ParseError::NotAnObject)
    }?;

    eprintln!("\n lseq: {} id: {}", record.lseq.unwrap(), record.id);
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
