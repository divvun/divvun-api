use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub addr: String,
}
