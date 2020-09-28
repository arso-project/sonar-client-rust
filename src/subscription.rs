use futures::stream::{Stream, StreamExt};
use log::*;
use std::future::Future;
use std::io::Error;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::schema;
use crate::Collection;

pub struct Subscription<'a> {
    name: String,
    collection: &'a Collection,
    last_cursor: Option<u64>,
    event_source: surf_sse::EventSource,
    state: SubState<'a>,
}
impl<'a> Subscription<'a> {
    pub fn new(collection: &'a Collection, name: String) -> Self {
        let event_source = collection.events().unwrap();
        Self {
            name,
            collection,
            last_cursor: None,
            event_source,
            state: SubState::Init,
        }
    }

    // async fn next(&mut self) -> http_types::Result<schema::PullResponse> {
    //     loop {
    //         if let Some(cursor) = self.last_cursor {
    //             self.collection.ack(&self.name, cursor).await?;
    //         }

    //         let res = self.collection.subscribe(&self.name).await?;
    //         self.last_cursor = Some(res.cursor);
    //         if res.messages.len() > 0 {
    //             return Ok(res);
    //         }
    //         // let mut events = self.collection.events()?;
    //         while let Some(event) = self.event_source.try_next().await? {
    //             eprintln!("event {:?}", event);
    //         }
    //     }
    //     // let schema::PullResponse {
    //     //     cursor,
    //     //     total,
    //     //     messages,
    //     // } = res;

    //     // let (tx, rx) = oneshot::channel();
    //     // let batch = Batch {
    //     //     ack: tx,
    //     //     records: messages,
    //     // };
    // }

    fn state_wait(&mut self) {
        self.state = SubState::Wait;
    }

    fn state_pull(&mut self) {
        let pull_fut = self.collection.pull_subscription(self.name.clone());
        let pull_fut = Box::pin(pull_fut);
        self.state = SubState::Pull(pull_fut);
    }

    fn state_ack(&mut self) {
        if let Some(cursor) = self.last_cursor {
            let fut = self.collection.ack_subscription(self.name.clone(), cursor);
            // let fut = fut.then(|r| r.map_err(|e| e.into()));
            self.state = SubState::Ack(Box::pin(fut));
        } else {
            self.state_pull();
        }
    }
}

type AckFuture<'a> = Pin<Box<dyn Future<Output = http_types::Result<()>> + Send + 'a>>;

type PullFuture<'a> =
    Pin<Box<dyn Future<Output = http_types::Result<schema::PullResponse>> + Send + 'a>>;

enum SubState<'a> {
    Init,
    Ack(AckFuture<'a>),
    Pull(PullFuture<'a>),
    Wait,
}

impl<'a> Stream for Subscription<'a> {
    type Item = http_types::Result<schema::PullResponse>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match &mut self.state {
                SubState::Init => {
                    self.state_pull();
                }
                SubState::Ack(fut) => {
                    let res = futures::ready!(Pin::new(fut).poll(cx));
                    res.unwrap();
                    self.state_pull();
                }
                SubState::Pull(fut) => {
                    let res = futures::ready!(Pin::new(fut).poll(cx));
                    match res {
                        Ok(batch) => {
                            self.last_cursor = Some(batch.cursor);
                            eprintln!("BATCH fin {:?}", batch.finished);
                            // TODO: Fix state machine for proper state transition.
                            self.state_wait();
                            // self.state_ack();
                            // if batch.finished {
                            // } else {
                            //     self.state_pull();
                            // }
                            // if batch.finished() {
                            //     self.state_wait();
                            // } else {
                            //     self.state_pull();
                            // }
                            if batch.messages.len() > 0 {
                                return Poll::Ready(Some(Ok(batch)));
                            }
                        }
                        Err(err) => return Poll::Ready(Some(Err(err))),
                    }
                }
                SubState::Wait => {
                    let poll = Pin::new(&mut self.event_source).poll_next(cx);
                    let event = futures::ready!(poll);
                    match event {
                        None => return Poll::Ready(None),
                        Some(Err(err)) => return Poll::Ready(Some(Err(err.into()))),
                        Some(Ok(event)) => {
                            eprintln!("event! {:?}", event);
                            self.state_ack();
                        }
                    }
                }
            }
        }
    }
}

// async fn subscribe(collection: &Collection, name: String) -> Result<(), Error> {
//     let mut sub = Subscription::new(collection.clone(), name);
//     while let Some(batch) = sub.next().await {
//         let batch = batch.unwrap();
//         on_batch(batch);
//     }
//     Ok(())
//     // loop {
//     //     let batch = sub.next().await.expect("subscription died");
//     //     on_batch(batch)?;
//     // }
//     // let res = collection.subscribe(name.clone()).await.unwrap();
//     // let cursor = res.cursor.clone();
//     // on_batch(res)?;
//     // collection.ack(name, cursor).await.unwrap();
//     // Ok(())
// }
