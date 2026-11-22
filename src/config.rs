use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub db: DbConfig,
    pub generation: GenerationConfig,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DbConfig {
    pub postgres_url: String,
    pub mongo_url: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GenerationConfig {
    pub seed: u64,
    pub matches_count: u32,
    pub pg_batch_size: usize,
    pub mongo_batch_size: usize,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}