use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub sentry_dsn: String,
    pub addr: String,
}
