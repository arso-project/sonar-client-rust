pub use sonar_hrpc::schema;

pub const DEFAULT_ENDPOINT: &'static str = "http://localhost:9191/api";

mod client;
mod collection;
mod subscription;

pub use client::Client;
pub use collection::Collection;
pub use subscription::Subscription;
