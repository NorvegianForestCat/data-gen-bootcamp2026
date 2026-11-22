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
/// It uses `ON CONFLICT DO NOTHING` to ensure idempotency across multiple runs.
///
/// # Arguments
///
/// * `pg_pool` - A reference to the PostgreSQL connection pool.
///
/// # Returns
///
/// * `Ok(())` if the reference data was successfully inserted or already exists.
/// * `Err(sqlx::Error)` if a database error occurs during the operation.
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
/// flushes it to the databases in batches to optimize I/O performance and
/// minimize connection overhead.
///
/// # Arguments
///
/// * `rng` - A mutable reference to a random number generator implementing `rand::Rng`.
/// * `pg_pool` - A reference to the PostgreSQL connection pool.
/// * `mongo_client` - A reference to the MongoDB client.
/// * `matches_count` - The total number of matches to generate.
/// * `batch_size` - The maximum number of records to accumulate before flushing to the database.
///
/// # Returns
///
/// * `Ok(())` if all matches and associated data were successfully generated and persisted.
/// * `Err(Box<dyn std::error::Error>)` if a database insertion or generation error occurs.
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
            if !player_batch.is_empty() {
                insert_player_batch(pg_pool, &player_batch).await?;
                player_batch.clear();
            }
            insert_match_batch(pg_pool, &match_batch).await?;
            match_batch.clear();

            tokio::try_join!(
                async {
                    if !match_player_batch.is_empty() {
                        insert_match_player_batch(pg_pool, &match_player_batch).await?;
                        match_player_batch.clear();
                    }
                    Ok::<(), Box<dyn std::error::Error>>(())
                },
                async {
                    if !mongo_batch.is_empty() {
                        mongo_collection.insert_many(&mongo_batch).await?;
                        mongo_batch.clear();
                    }
                    Ok::<(), Box<dyn std::error::Error>>(())
                }
            )?;
        }
    }

    // Flush remaining records
    if !player_batch.is_empty() {
        insert_player_batch(pg_pool, &player_batch).await?;
    }
    if !match_batch.is_empty() {
        insert_match_batch(pg_pool, &match_batch).await?;
    }

    tokio::try_join!(
        async {
            if !match_player_batch.is_empty() {
                insert_match_player_batch(pg_pool, &match_player_batch).await?;
            }
            Ok::<(), Box<dyn std::error::Error>>(())
        },
        async {
            if !mongo_batch.is_empty() {
                mongo_collection.insert_many(&mongo_batch).await?;
            }
            Ok::<(), Box<dyn std::error::Error>>(())
        }
    )?;

    Ok(())
}

/// Inserts a batch of match records into the PostgreSQL database.
///
/// This function uses PostgreSQL's `UNNEST` with array parameters for efficient
/// bulk insertion. It handles `ON CONFLICT (match_id) DO NOTHING` to ensure
/// idempotency.
///
/// # Arguments
///
/// * `pg_pool` - A reference to the PostgreSQL connection pool.
/// * `batch` - A slice of `MatchRecord` instances to insert.
///
/// # Returns
///
/// * `Ok(())` if the batch was successfully inserted.
/// * `Err(sqlx::Error)` if a database error occurs.
async fn insert_match_batch(pg_pool: &PgPool, batch: &[MatchRecord]) -> Result<(), sqlx::Error> {
    if batch.is_empty() { return Ok(()); }

    let mut match_id = Vec::with_capacity(batch.len());
    let mut match_seq_num = Vec::with_capacity(batch.len());
    let mut start_time = Vec::with_capacity(batch.len());
    let mut duration = Vec::with_capacity(batch.len());
    let mut game_mode = Vec::with_capacity(batch.len());
    let mut lobby_type = Vec::with_capacity(batch.len());
    let mut radiant_win = Vec::with_capacity(batch.len());
    let mut radiant_score = Vec::with_capacity(batch.len());
    let mut dire_score = Vec::with_capacity(batch.len());
    let mut tower_status_radiant = Vec::with_capacity(batch.len());
    let mut tower_status_dire = Vec::with_capacity(batch.len());
    let mut barracks_status_radiant = Vec::with_capacity(batch.len());
    let mut barracks_status_dire = Vec::with_capacity(batch.len());
    let mut cluster = Vec::with_capacity(batch.len());
    let mut first_blood_time = Vec::with_capacity(batch.len());
    let mut human_players = Vec::with_capacity(batch.len());

    for m in batch {
        match_id.push(m.match_id);
        match_seq_num.push(m.match_seq_num);
        start_time.push(m.start_time as i64);
        duration.push(m.duration);
        game_mode.push(m.game_mode);
        lobby_type.push(m.lobby_type);
        radiant_win.push(m.radiant_win);
        radiant_score.push(m.radiant_score);
        dire_score.push(m.dire_score);
        tower_status_radiant.push(m.tower_status_radiant);
        tower_status_dire.push(m.tower_status_dire);
        barracks_status_radiant.push(m.barracks_status_radiant);
        barracks_status_dire.push(m.barracks_status_dire);
        cluster.push(m.cluster);
        first_blood_time.push(m.first_blood_time);
        human_players.push(m.human_players);
    }

    sqlx::query(
        "INSERT INTO matches (
            match_id, match_seq_num, start_time, duration, game_mode, lobby_type,
            radiant_win, radiant_score, dire_score, tower_status_radiant, tower_status_dire,
            barracks_status_radiant, barracks_status_dire, cluster, first_blood_time, human_players,
            loaded_at, stratz_parsed_at, discover_run_id, enrich_run_id, enrichment_attempts
         )
         SELECT *, NOW(), NOW(), NULL, NULL, 0
         FROM UNNEST(
            $1::bigint[], $2::bigint[], $3::bigint[], $4::int[], $5::int[], $6::int[],
            $7::bool[], $8::int[], $9::int[], $10::int[], $11::int[],
            $12::int[], $13::int[], $14::int[], $15::int[], $16::int[]
         )
         ON CONFLICT (match_id) DO NOTHING"
    )
    .bind(&match_id).bind(&match_seq_num).bind(&start_time).bind(&duration)
    .bind(&game_mode).bind(&lobby_type).bind(&radiant_win).bind(&radiant_score)
    .bind(&dire_score).bind(&tower_status_radiant).bind(&tower_status_dire)
    .bind(&barracks_status_radiant).bind(&barracks_status_dire).bind(&cluster)
    .bind(&first_blood_time).bind(&human_players)
    .execute(pg_pool).await?;

    Ok(())
}

/// Inserts a batch of unique player records into the PostgreSQL database.
///
/// This function performs an upsert operation (`ON CONFLICT DO UPDATE`) to ensure
/// that player names are kept up-to-date while avoiding duplicate entries.
///
/// # Arguments
///
/// * `pg_pool` - A reference to the PostgreSQL connection pool.
/// * `batch` - A slice of `PlayerRecord` instances to insert or update.
///
/// # Returns
///
/// * `Ok(())` if the batch was successfully processed.
/// * `Err(sqlx::Error)` if a database error occurs.
async fn insert_player_batch(pg_pool: &PgPool, batch: &[PlayerRecord]) -> Result<(), sqlx::Error> {
    if batch.is_empty() { return Ok(()); }

    let account_ids: Vec<i64> = batch.iter().map(|p| p.account_id).collect();
    let personanames: Vec<String> = batch.iter().map(|p| p.personaname.clone()).collect();

    sqlx::query(
        "INSERT INTO players (account_id, personaname, loaded_at)
         SELECT *, NOW() FROM UNNEST($1::bigint[], $2::text[])
         ON CONFLICT (account_id) DO UPDATE SET personaname = EXCLUDED.personaname, loaded_at = NOW()"
    )
    .bind(&account_ids)
    .bind(&personanames)
    .execute(pg_pool).await?;

    Ok(())
}

/// Inserts a batch of match-player statistics into the PostgreSQL database.
///
/// This function uses PostgreSQL's `UNNEST` for efficient bulk insertion of
/// player performance metrics per match. It ignores conflicts on the composite
/// primary key `(match_id, player_slot)` to ensure idempotency.
///
/// # Arguments
///
/// * `pg_pool` - A reference to the PostgreSQL connection pool.
/// * `batch` - A slice of `MatchPlayerRecord` instances to insert.
///
/// # Returns
///
/// * `Ok(())` if the batch was successfully inserted.
/// * `Err(sqlx::Error)` if a database error occurs.
async fn insert_match_player_batch(pg_pool: &PgPool, batch: &[MatchPlayerRecord]) -> Result<(), sqlx::Error> {
    if batch.is_empty() { return Ok(()); }

    let mut match_id = Vec::with_capacity(batch.len());
    let mut player_slot = Vec::with_capacity(batch.len());
    let mut account_id = Vec::with_capacity(batch.len());
    let mut hero_id = Vec::with_capacity(batch.len());
    let mut item_0 = Vec::with_capacity(batch.len());
    let mut item_1 = Vec::with_capacity(batch.len());
    let mut item_2 = Vec::with_capacity(batch.len());
    let mut item_3 = Vec::with_capacity(batch.len());
    let mut item_4 = Vec::with_capacity(batch.len());
    let mut item_5 = Vec::with_capacity(batch.len());
    let mut backpack_0 = Vec::with_capacity(batch.len());
    let mut backpack_1 = Vec::with_capacity(batch.len());
    let mut backpack_2 = Vec::with_capacity(batch.len());
    let mut item_neutral = Vec::with_capacity(batch.len());
    let mut kills = Vec::with_capacity(batch.len());
    let mut deaths = Vec::with_capacity(batch.len());
    let mut assists = Vec::with_capacity(batch.len());
    let mut kda = Vec::with_capacity(batch.len());
    let mut gold = Vec::with_capacity(batch.len());
    let mut gold_per_min = Vec::with_capacity(batch.len());
    let mut xp_per_min = Vec::with_capacity(batch.len());
    let mut gold_spent = Vec::with_capacity(batch.len());
    let mut net_worth = Vec::with_capacity(batch.len());
    let mut hero_damage = Vec::with_capacity(batch.len());
    let mut tower_damage = Vec::with_capacity(batch.len());
    let mut hero_healing = Vec::with_capacity(batch.len());
    let mut level = Vec::with_capacity(batch.len());
    let mut last_hits = Vec::with_capacity(batch.len());
    let mut denies = Vec::with_capacity(batch.len());
    let mut is_radiant = Vec::with_capacity(batch.len());
    let mut win = Vec::with_capacity(batch.len());
    let mut aghanims_scepter = Vec::with_capacity(batch.len());
    let mut aghanims_shard = Vec::with_capacity(batch.len());
    let mut rank = Vec::with_capacity(batch.len());

    for mp in batch {
        match_id.push(mp.match_id);
        player_slot.push(mp.player_slot);
        account_id.push(mp.account_id);
        hero_id.push(mp.hero_id);
        item_0.push(mp.item_0);
        item_1.push(mp.item_1);
        item_2.push(mp.item_2);
        item_3.push(mp.item_3);
        item_4.push(mp.item_4);
        item_5.push(mp.item_5);
        backpack_0.push(mp.backpack_0);
        backpack_1.push(mp.backpack_1);
        backpack_2.push(mp.backpack_2);
        item_neutral.push(mp.item_neutral);
        kills.push(mp.kills);
        deaths.push(mp.deaths);
        assists.push(mp.assists);
        kda.push(mp.kda as f64);
        gold.push(mp.gold);
        gold_per_min.push(mp.gold_per_min);
        xp_per_min.push(mp.xp_per_min);
        gold_spent.push(mp.gold_spent);
        net_worth.push(mp.net_worth);
        hero_damage.push(mp.hero_damage);
        tower_damage.push(mp.tower_damage);
        hero_healing.push(mp.hero_healing);
        level.push(mp.level);
        last_hits.push(mp.last_hits);
        denies.push(mp.denies);
        is_radiant.push(mp.is_radiant);
        win.push(mp.win);
        aghanims_scepter.push(mp.aghanims_scepter);
        aghanims_shard.push(mp.aghanims_shard);
        rank.push(mp.rank);
    }

    sqlx::query(
        "INSERT INTO match_players (
            match_id, player_slot, account_id, hero_id, item_0, item_1, item_2, item_3, item_4, item_5,
            backpack_0, backpack_1, backpack_2, item_neutral, kills, deaths, assists, kda, gold,
            gold_per_min, xp_per_min, gold_spent, net_worth, hero_damage, tower_damage, hero_healing,
            level, last_hits, denies, is_radiant, win, aghanims_scepter, aghanims_shard, rank, loaded_at
         )
         SELECT *, NOW()
         FROM UNNEST(
            $1::bigint[], $2::int[], $3::bigint[], $4::int[], $5::int[], $6::int[], $7::int[], $8::int[], $9::int[], $10::int[],
            $11::int[], $12::int[], $13::int[], $14::int[], $15::int[], $16::int[], $17::int[], $18::float8[], $19::int[],
            $20::int[], $21::int[], $22::int[], $23::int[], $24::int[], $25::int[], $26::int[],
            $27::int[], $28::int[], $29::int[], $30::bool[], $31::bool[], $32::bool[], $33::bool[], $34::int[]
         )
         ON CONFLICT (match_id, player_slot) DO NOTHING"
    )
    .bind(&match_id).bind(&player_slot).bind(&account_id).bind(&hero_id)
    .bind(&item_0).bind(&item_1).bind(&item_2).bind(&item_3).bind(&item_4).bind(&item_5)
    .bind(&backpack_0).bind(&backpack_1).bind(&backpack_2).bind(&item_neutral)
    .bind(&kills).bind(&deaths).bind(&assists).bind(&kda).bind(&gold)
    .bind(&gold_per_min).bind(&xp_per_min).bind(&gold_spent).bind(&net_worth)
    .bind(&hero_damage).bind(&tower_damage).bind(&hero_healing)
    .bind(&level).bind(&last_hits).bind(&denies).bind(&is_radiant).bind(&win)
    .bind(&aghanims_scepter).bind(&aghanims_shard).bind(&rank)
    .execute(pg_pool).await?;

    Ok(())
}