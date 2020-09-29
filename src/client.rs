// use anyhow::Result;
use http_types::Result;
pub use sonar_hrpc::schema;

use crate::{Collection, DEFAULT_ENDPOINT};

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
        let res = surf::get(url).await?;
        Ok(res)
    }

    pub fn url(&self, path: impl ToString) -> String {
        format!("{}{}", self.endpoint, path.to_string())
    }

    pub fn collection(&self, name: impl ToString) -> Collection {
        Collection::new(self.clone(), name.to_string())
    }
}
