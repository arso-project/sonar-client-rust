// use anyhow::Result;
use http_types::{Method, Result};
pub use sonar_hrpc::schema;
use std::io::Result as IoResult;
use surf_sse::{Error, EventSource};

const DEFAULT_ENDPOINT: &'static str = "http://localhost:9191/api";

mod subscription;
use subscription::Subscription;

#[derive(Clone)]
pub struct Client {
    endpoint: String,
}

impl Default for Client {
    fn default() -> Self {
        Client::new(DEFAULT_ENDPOINT)
    }
}

impl Client {
    pub fn new(endpoint: impl ToString) -> Self {
        Self {
            endpoint: endpoint.to_string(),
        }
    }

    pub async fn get(&self, path: impl ToString) -> Result<surf::Response> {
        let url = self.url(path);
        let mut res = surf::get(url).await?;
        // println!("{}", res.body_string().await?);
        // eprintln!("YAY {:?}", info);
        Ok(res)
    }

    pub fn url(&self, path: impl ToString) -> String {
        format!("{}{}", self.endpoint, path.to_string())
    }

    //     pub fn fetch(&self, path: impl ToString) -> surf::Response

    //     pub async fn collection(&mut self, name: impl ToString) -> http_types::Result<()> {
    //         let mut res = surf::get(self.url(name)).await?;
    //         // let collection = Collection::new(&self);
    //         // collection.open().await;
    //         // eprintln!("res {:?}", res.body_string().await);
    //         let info: IoResult<CollectionInfo> = res.body_json().await;
    //         eprintln!("info {:?}", info);
    //     }
}

#[derive(Clone)]
pub struct Collection {
    name: String,
    client: Client,
}
impl Collection {
    pub fn new(client: Client, name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            client,
        }
    }

    // pub async fn get(&self, path: impl ToString) -> Result<surf::Response> {
    //     let url = self.url(path);
    //     self.client.get(url).await
    // }

    pub fn url(&self, path: impl ToString) -> String {
        let url = self
            .client
            .url(format!("/collection/{}{}", self.name, path.to_string()).to_string());
        eprintln!("url {}", url);
        url
    }

    pub fn subscribe(&self, name: impl ToString) -> Subscription {
        Subscription::new(self, name.to_string())
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn pull_subscription(&self, name: impl ToString) -> Result<schema::PullResponse> {
        let url = self.url(format!("/subscription/{}", name.to_string()).to_string());
        let mut res = surf::get(url).await?;
        let info: IoResult<schema::PullResponse> = res.body_json().await;
        Ok(info?)
    }

    pub async fn ack_subscription(&self, name: impl ToString, cursor: u64) -> Result<()> {
        let url = self.url(self.url(format!("/subscription/{}/{}", name.to_string(), cursor)));
        let _res = surf::post(url).await?;
        Ok(())
    }

    pub fn events(&self) -> Result<EventSource> {
        let stream = EventSource::new(self.url("/events").parse()?);
        Ok(stream)
    }
}
