//! Data generation orchestration and batch insertion logic.
//!
//! This module coordinates the generation of matches, players, and events,
//! and manages batched inserts into PostgreSQL and MongoDB to optimize
//! performance and reduce database load.

pub mod events;
pub mod matches;
pub mod player;

use sqlx::PgPool;
use mongodb::Client;
use crate::generators::matches::{generate_match, MatchRecord};
use crate::generators::player::{PlayerRecord, MatchPlayerRecord};
use std::collections::HashSet;

/// Inserts foundational reference data into the PostgreSQL database.
///
/// This function populates the `heroes` and `npcs` tables with baseline
/// records required for match generation, such as the Roshan NPC (ID 133).
/// It uses `ON CONFLICT DO NOTHING` to ensure idempotency.
///
/// # Arguments
///
/// * `pg_pool` - A reference to the PostgreSQL connection pool.
///
/// # Returns
///
/// A `Result` indicating success or a `sqlx::Error` on failure.
pub async fn create_reference_data(pg_pool: &PgPool) -> Result<(), sqlx::Error> {
    // Heroes
    sqlx::query(
        "INSERT INTO heroes (hero_id, name, localized_name, primary_attr, attack_type, roles)
         VALUES (1, 'npc_dota_hero_antimage', 'Anti-Mage', 'agi', 'Melee', '{Carry,Escape,Nuker}'),
                (2, 'npc_dota_hero_axe', 'Axe', 'str', 'Melee', '{Initiator,Durable,Disabler}')
         ON CONFLICT (hero_id) DO NOTHING"
    ).execute(pg_pool).await?;

    // NPCs (Roshan)
    sqlx::query(
        "INSERT INTO npcs (npc_id, name, status_health, status_mana, movement_speed, attack_range, attack_damage_min, attack_damage_max, attack_rate, projectile_speed, team_name, is_neutral_unit, is_ancient, can_be_dominated, has_inventory, level)
         VALUES (133, 'npc_dota_roshan', 6000.0, 0.0, 100.0, 150.0, 75.0, 85.0, 1.0, 1000.0, 'Neutrals', true, true, false, false, 1.0)
         ON CONFLICT (npc_id) DO NOTHING"
    ).execute(pg_pool).await?;

    Ok(())
}

/// Generates and persists a specified number of matches and their associated data.
///
/// This function orchestrates the creation of match records, player records,
/// and MongoDB event documents. It accumulates generated data in memory and
/// flushes it to the databases in batches to optimize I/O performance.
///
/// # Arguments
///
/// * `rng` - A mutable reference to a random number generator.
/// * `pg_pool` - A reference to the PostgreSQL connection pool.
/// * `mongo_client` - A reference to the MongoDB client.
/// * `matches_count` - The total number of matches to generate.
/// * `batch_size` - The maximum number of records to accumulate before flushing.
///
/// # Returns
///
/// A `Result` indicating success or a boxed error on failure.
pub async fn generate_matches(
    rng: &mut impl rand::Rng,
    pg_pool: &PgPool,
    mongo_client: &Client,
    matches_count: usize,
    batch_size: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mongo_collection = mongo_client.database("dota_events").collection::<bson::Document>("cs_events");
    
    let mut match_batch = Vec::new();
    let mut player_batch = Vec::new();
    let mut match_player_batch = Vec::new();
    let mut mongo_batch = Vec::new();
    let mut seen_players = HashSet::new();
    let mut match_seq_num = 1_000_000_000i64;

    for i in 0..matches_count {
        let match_id = 7_000_000_000i64 + i as i64;
        let generated = generate_match(rng, match_id, match_seq_num);
        match_seq_num += 1;

        match_batch.push(generated.match_record);
        for p in generated.unique_players {
            if seen_players.insert(p.account_id) {
                player_batch.push(p);
            }
        }
        for mp in generated.player_records {
            match_player_batch.push(mp);
        }
        for event in generated.mongo_events {
            mongo_batch.push(event);
        }

        if match_batch.len() >= batch_size {
            insert_match_batch(pg_pool, &match_batch).await?;
            match_batch.clear();
        }
        if player_batch.len() >= batch_size {
            insert_player_batch(pg_pool, &player_batch).await?;
            player_batch.clear();
        }
        if match_player_batch.len() >= batch_size {
            insert_match_player_batch(pg_pool, &match_player_batch).await?;
            match_player_batch.clear();
        }
        if mongo_batch.len() >= batch_size {
            mongo_collection.insert_many(&mongo_batch).await?;
            mongo_batch.clear();
        }
    }

    if !match_batch.is_empty() {
        insert_match_batch(pg_pool, &match_batch).await?;
    }
    if !player_batch.is_empty() {
        insert_player_batch(pg_pool, &player_batch).await?;
    }
    if !match_player_batch.is_empty() {
        insert_match_player_batch(pg_pool, &match_player_batch).await?;
    }
    if !mongo_batch.is_empty() {
        mongo_collection.insert_many(&mongo_batch).await?;
    }

    Ok(())
}

/// Inserts a batch of match records into the PostgreSQL database.
///
/// # Arguments
///
/// * `pg_pool` - A reference to the PostgreSQL connection pool.
/// * `batch` - A slice of `MatchRecord` instances to insert.
///
/// # Returns
///
/// A `Result` indicating success or a `sqlx::Error` on failure.
async fn insert_match_batch(pg_pool: &PgPool, batch: &[MatchRecord]) -> Result<(), sqlx::Error> {
    for m in batch {
        sqlx::query(
            "INSERT INTO matches (match_id, match_seq_num, start_time, duration, game_mode, lobby_type, radiant_win, radiant_score, dire_score, tower_status_radiant, tower_status_dire, barracks_status_radiant, barracks_status_dire, cluster, first_blood_time, human_players, loaded_at, stratz_parsed_at, discover_run_id, enrich_run_id, enrichment_attempts)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, NOW(), NOW(), NULL, NULL, 0)"
        )
        .bind(m.match_id)
        .bind(m.match_seq_num)
        .bind(m.start_time)
        .bind(m.duration)
        .bind(m.game_mode)
        .bind(m.lobby_type)
        .bind(m.radiant_win)
        .bind(m.radiant_score)
        .bind(m.dire_score)
        .bind(m.tower_status_radiant)
        .bind(m.tower_status_dire)
        .bind(m.barracks_status_radiant)
        .bind(m.barracks_status_dire)
        .bind(m.cluster)
        .bind(m.first_blood_time)
        .bind(m.human_players)
        .execute(pg_pool).await?;
    }
    Ok(())
}

/// Inserts a batch of player records into the PostgreSQL database.
///
/// Uses `ON CONFLICT DO UPDATE` to ensure idempotent upserts based on `account_id`.
///
/// # Arguments
///
/// * `pg_pool` - A reference to the PostgreSQL connection pool.
/// * `batch` - A slice of `PlayerRecord` instances to insert.
///
/// # Returns
///
/// A `Result` indicating success or a `sqlx::Error` on failure.
async fn insert_player_batch(pg_pool: &PgPool, batch: &[PlayerRecord]) -> Result<(), sqlx::Error> {
    for p in batch {
        sqlx::query(
            "INSERT INTO players (account_id, personaname, loaded_at)
             VALUES ($1, $2, NOW())
             ON CONFLICT (account_id) DO UPDATE SET personaname = EXCLUDED.personaname, loaded_at = NOW()"
        )
        .bind(p.account_id)
        .bind(p.personaname.clone())
        .execute(pg_pool).await?;
    }
    Ok(())
}

/// Inserts a batch of match-player records into the PostgreSQL database.
///
/// Uses `ON CONFLICT DO NOTHING` to prevent duplicate entries for the same
/// `(match_id, player_slot)` combination.
///
/// # Arguments
///
/// * `pg_pool` - A reference to the PostgreSQL connection pool.
/// * `batch` - A slice of `MatchPlayerRecord` instances to insert.
///
/// # Returns
///
/// A `Result` indicating success or a `sqlx::Error` on failure.
async fn insert_match_player_batch(pg_pool: &PgPool, batch: &[MatchPlayerRecord]) -> Result<(), sqlx::Error> {
    for mp in batch {
        sqlx::query(
            "INSERT INTO match_players (match_id, player_slot, account_id, hero_id, item_0, item_1, item_2, item_3, item_4, item_5, backpack_0, backpack_1, backpack_2, item_neutral, kills, deaths, assists, kda, gold, gold_per_min, xp_per_min, gold_spent, net_worth, hero_damage, tower_damage, hero_healing, level, last_hits, denies, is_radiant, win, aghanims_scepter, aghanims_shard, rank, loaded_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32, $33, $34, NOW())
             ON CONFLICT (match_id, player_slot) DO NOTHING"
        )
        .bind(mp.match_id)
        .bind(mp.player_slot)
        .bind(mp.account_id)
        .bind(mp.hero_id)
        .bind(mp.item_0)
        .bind(mp.item_1)
        .bind(mp.item_2)
        .bind(mp.item_3)
        .bind(mp.item_4)
        .bind(mp.item_5)
        .bind(mp.backpack_0)
        .bind(mp.backpack_1)
        .bind(mp.backpack_2)
        .bind(mp.item_neutral)
        .bind(mp.kills)
        .bind(mp.deaths)
        .bind(mp.assists)
        .bind(mp.kda)
        .bind(mp.gold)
        .bind(mp.gold_per_min)
        .bind(mp.xp_per_min)
        .bind(mp.gold_spent)
        .bind(mp.net_worth)
        .bind(mp.hero_damage)
        .bind(mp.tower_damage)
        .bind(mp.hero_healing)
        .bind(mp.level)
        .bind(mp.last_hits)
        .bind(mp.denies)
        .bind(mp.is_radiant)
        .bind(mp.win)
        .bind(mp.aghanims_scepter)
        .bind(mp.aghanims_shard)
        .bind(mp.rank)
        .execute(pg_pool).await?;
    }
    Ok(())
}
