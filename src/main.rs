use clap::Parser;
use std::path::PathBuf;


#[derive(Parser, Debug)]
#[command(name = "dota-data-generator")]
#[command(about = "Dota 2 Data Generator for Bootcamp 2026")]
struct Args {
    #[arg(short, long, default_value = "config/default.toml")]
    config: PathBuf,

    #[arg(short, long)]
    matches: Option<u32>,

    #[arg(short, long)]
    seed: Option<u64>,
}

mod config;
mod db;
mod generators;
mod rng;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut cfg = config::Config::load(&args.config)?;

    if let Some(m) = args.matches {
        cfg.generation.matches_count = m;
    }
    if let Some(s) = args.seed {
        cfg.generation.seed = s;
    }

    println!("Starting generator with {} matches, seed {}", cfg.generation.matches_count, cfg.generation.seed);

    let pg_pool = db::postgres::create_pool(&cfg.db.postgres_url).await?;
    let mongo_client = db::mongo::create_client(&cfg.db.mongo_url).await?;

    println!("Creating reference data...");
    generators::create_reference_data(&pg_pool).await?;

    println!("Generating {} matches...", cfg.generation.matches_count);
    let mut rng = rng::create_rng(cfg.generation.seed);
    generators::generate_matches(
        &mut rng,
        &pg_pool,
        &mongo_client,
        cfg.generation.matches_count as usize,
        cfg.generation.pg_batch_size,
    ).await?;

    println!("Generation completed successfully!");
    Ok(())
}
