use mongodb::{Client, options::ClientOptions};

pub async fn create_client(url: &str) -> Result<Client, mongodb::error::Error> {
    let client_options = ClientOptions::parse(url).await?;
    Client::with_options(client_options)
}