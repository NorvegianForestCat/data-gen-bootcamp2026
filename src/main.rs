use clap::Parser;
use std::path::PathBuf;


#[derive(Parser, Debug)]
#[command(name = "dota-data-generator")]
#[command(about = "Dota 2 Data Generator for Bootcamp 2026")]
struct Args {
    #[arg(long, env = "POSTGRES_URL")]
    postgres_url: String,

    #[arg(long, env = "MONGO_URL")]
    mongo_url: String,

    #[arg(long, env = "MATCHES", default_value_t = 1000)]
    matches_count: usize,

    #[arg(long, env = "SEED", default_value_t = 42)]
    seed: u64,
}

mod config;
mod db;
mod generators;
mod rng;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let postgres_url = args.postgres_url;
    let mongo_url = args.mongo_url;
    let matches_count = args.matches_count;
    let seed = args.seed;

    println!("Starting generator with {} matches, seed {}", matches_count, seed);

    let pg_pool = db::postgres::create_pool(&postgres_url).await?;
    let mongo_client = db::mongo::create_client(&mongo_url).await?;

    sqlx::migrate!("./migrations").run(&pg_pool).await?;

    println!("Creating reference data...");
    generators::create_reference_data(&pg_pool).await?;

    println!("Generating {} matches...", matches_count);
    let mut rng = rng::create_rng(seed);
    generators::generate_matches(
        &mut rng,
        &pg_pool,
        &mongo_client,
        matches_count,
        1000,
    ).await?;

    println!("Generation completed successfully!");
    Ok(())
}
