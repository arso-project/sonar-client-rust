use http_types::Result;
use log::*;
use serde::Serialize;
use sonar_hrpc::schema;
use surf_sse::EventSource;

use crate::{Client, Subscription};

#[derive(Clone)]
pub struct Collection {
    name: String,
    client: Client,
}

// #[derive(Serialize,Deserialize)]
pub type QueryResponse = Vec<schema::Record>;

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
        debug!("url {}", url);
        url
    }

    pub async fn query(&self, name: &str, args: impl Serialize) -> Result<QueryResponse> {
        let url = self.url(&format!("/query/{}", name));
        let body = surf::Body::from_json(&args)?;
        let mut res = surf::post(url).body(body).await?;
        res.body_json().await
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
        res.body_json().await
    }

    pub async fn ack_subscription(&self, name: impl ToString, cursor: u64) -> Result<()> {
        let url = self.url(format!("/subscription/{}/{}", name.to_string(), cursor));
        let _res = surf::post(url).await?;
        Ok(())
    }

    pub fn events(&self) -> Result<EventSource> {
        let stream = EventSource::new(self.url("/events").parse()?);
        Ok(stream)
    }
}
