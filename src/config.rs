//! Configuration management for the Dota 2 Data Generator.
//!
//! This module provides structures and methods for loading configuration
//! settings from a TOML file, including database connection strings and
//! generation parameters.

#![deprecated(since = "0.1.2", note = "This module is deprecated and will be removed in future versions.")]

use serde::Deserialize;
use std::path::Path;

/// Root configuration structure containing database and generation settings.
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    /// Database connection configuration.
    pub db: DbConfig,
    /// Data generation parameters.
    pub generation: GenerationConfig,
}

/// Configuration for database connections.
#[derive(Deserialize, Debug, Clone)]
pub struct DbConfig {
    /// PostgreSQL connection URL.
    pub postgres_url: String,
    /// MongoDB connection URL.
    pub mongo_url: String,
}

/// Configuration for data generation behavior.
#[derive(Deserialize, Debug, Clone)]
pub struct GenerationConfig {
    /// Random seed for reproducible generation.
    pub seed: u64,
    /// Total number of matches to generate.
    pub matches_count: u32,
    /// Batch size for PostgreSQL inserts.
    pub pg_batch_size: usize,
    /// Batch size for MongoDB inserts.
    pub mongo_batch_size: usize,
}

impl Config {
    /// Loads configuration from a TOML file at the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the TOML configuration file.
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `Config` on success, or a boxed error
    /// if the file cannot be read or parsed.
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
