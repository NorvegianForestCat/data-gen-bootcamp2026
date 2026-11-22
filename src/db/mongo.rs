//! MongoDB connection utilities.
//!
//! This module provides functions for establishing a client connection
//! to a MongoDB database using the official `mongodb` crate.

use mongodb::{Client, options::ClientOptions};

/// Creates a new `mongodb::Client` from a MongoDB connection URL.
///
/// # Arguments
///
/// * `url` – A MongoDB connection string, e.g., `"mongodb://localhost:27017"`.
///
/// # Returns
///
/// A `Result` containing either the initialized `Client` or a `mongodb::error::Error`
/// if the URL could not be parsed or the client could not be constructed.
///
/// # Example
///
/// ```ignore
/// let client = create_client("mongodb://localhost:27017").await?;
/// ```
pub async fn create_client(url: &str) -> Result<Client, mongodb::error::Error> {
    let client_options = ClientOptions::parse(url).await?;
    Client::with_options(client_options)
}
