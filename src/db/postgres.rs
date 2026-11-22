use sqlx::postgres::PgPoolOptions;

pub async fn create_pool(url: &str) -> Result<sqlx::PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(url)
        .await
}