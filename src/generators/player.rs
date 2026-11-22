//! Player and match-player record generation logic.
//!
//! This module provides functions to generate synthetic player profiles
//! and their corresponding in-match statistics, ensuring data conforms
//! to the expected database schema.

use rand::Rng;

/// Represents a unique player profile.
#[derive(Clone)]
pub struct PlayerRecord {
    /// Unique Steam account identifier.
    pub account_id: i64,
    /// Display name of the player.
    pub personaname: String,
}

/// Represents a player's statistics and state within a specific match.
#[derive(Clone)]
pub struct MatchPlayerRecord {
    /// Unique identifier of the match.
    pub match_id: i64,
    /// Slot identifier (0-4 for Radiant, 128-132 for Dire).
    pub player_slot: i32,
    /// Unique Steam account identifier of the player.
    pub account_id: i64,
    /// Identifier of the hero played.
    pub hero_id: i32,
    /// Item in slot 0.
    pub item_0: Option<i32>,
    /// Item in slot 1.
    pub item_1: Option<i32>,
    /// Item in slot 2.
    pub item_2: Option<i32>,
    /// Item in slot 3.
    pub item_3: Option<i32>,
    /// Item in slot 4.
    pub item_4: Option<i32>,
    /// Item in slot 5.
    pub item_5: Option<i32>,
    /// Item in backpack slot 0.
    pub backpack_0: Option<i32>,
    /// Item in backpack slot 1.
    pub backpack_1: Option<i32>,
    /// Item in backpack slot 2.
    pub backpack_2: Option<i32>,
    /// Neutral item equipped.
    pub item_neutral: Option<i32>,
    /// Total number of kills.
    pub kills: i32,
    /// Total number of deaths.
    pub deaths: i32,
    /// Total number of assists.
    pub assists: i32,
    /// Kill-Death-Assist ratio.
    pub kda: f32,
    /// Gold held at the end of the match.
    pub gold: i32,
    /// Average gold earned per minute.
    pub gold_per_min: i32,
    /// Average experience earned per minute.
    pub xp_per_min: i32,
    /// Total gold spent during the match.
    pub gold_spent: i32,
    /// Total net worth (gold + gold_spent).
    pub net_worth: i32,
    /// Total damage dealt to heroes.
    pub hero_damage: i32,
    /// Total damage dealt to towers.
    pub tower_damage: i32,
    /// Total healing provided to allies.
    pub hero_healing: i32,
    /// Final hero level.
    pub level: i32,
    /// Total number of last hits.
    pub last_hits: i32,
    /// Total number of denies.
    pub denies: i32,
    /// Whether the player was on the Radiant team.
    pub is_radiant: bool,
    /// Whether the player's team won the match.
    pub win: bool,
    /// Whether the player purchased Aghanim's Scepter.
    pub aghanims_scepter: bool,
    /// Whether the player purchased Aghanim's Shard.
    pub aghanims_shard: bool,
    /// Player's MMR rank bracket (1-80).
    pub rank: i32,
}

/// Generates a synthetic player profile.
///
/// # Arguments
///
/// * `rng` - A mutable reference to a random number generator.
/// * `account_id` - The unique account identifier to assign to this player.
///
/// # Returns
///
/// A `PlayerRecord` with a generated `personaname`.
pub fn generate_player_record(rng: &mut impl Rng, account_id: i64) -> PlayerRecord {
    let prefixes = ["xX", "Pro", "Noob", "Dota", "Shadow", "Dark", "Light", "Super"];
    let suffixes = ["_gamer", "123", "OP", "GG", "_dota", "killer", "master", "pro"];
    
    let prefix = prefixes[rng.gen_range(0..prefixes.len())];
    let suffix = suffixes[rng.gen_range(0..suffixes.len())];
    let num = rng.gen_range(1000..9999);
    
    let personaname = if rng.gen_bool(0.1) {
        format!("Player_{}", account_id)
    } else {
        format!("{}{}{}{}", prefix, suffix, if rng.gen_bool(0.5) { "_" } else { "" }, num)
    };

    PlayerRecord {
        account_id,
        personaname,
    }
}

/// Generates synthetic statistics for a player within a specific match.
///
/// # Arguments
///
/// * `rng` - A mutable reference to a random number generator.
/// * `match_id` - The unique identifier of the match.
/// * `player_slot` - The player's slot (0-4 or 128-132).
/// * `duration` - The total duration of the match in seconds.
/// * `radiant_win` - Whether the Radiant team won the match.
/// * `is_radiant` - Whether this specific player is on the Radiant team.
///
/// # Returns
///
/// A `MatchPlayerRecord` populated with randomized but realistic statistics.
pub fn generate_match_player_record(
    rng: &mut impl Rng,
    match_id: i64,
    player_slot: i32,
    duration: i32,
    radiant_win: bool,
    is_radiant: bool,
) -> MatchPlayerRecord {
    let account_id = rng.gen_range(100_000_000..1_000_000_000);
    let hero_id = rng.gen_range(1..=124);
    
    let kills = rng.gen_range(0..=25);
    let deaths = rng.gen_range(0..=20);
    let assists = rng.gen_range(0..=35);
    let kda = if deaths == 0 {
        (kills + assists) as f32
    } else {
        (kills + assists) as f32 / deaths as f32
    };
    
    let duration_min = (duration / 60) as f32;
    let gpm = rng.gen_range(200..=1000);
    let xpm = rng.gen_range(200..=1200);
    let gold = rng.gen_range(0..=5000);
    let gold_spent = (gpm as f32 * duration_min * rng.gen_range(70..=95) as f32 / 100.0) as i32;
    let net_worth = gold + gold_spent;
    
    let hero_damage = rng.gen_range(5000..=60000);
    let tower_damage = rng.gen_range(0..=15000);
    let hero_healing = rng.gen_range(0..=20000);
    
    let level = rng.gen_range(1..=30);
    let last_hits = rng.gen_range(0..=600);
    let denies = rng.gen_range(0..=50);
    
    let rank = rng.gen_range(1..=80);
    let mut item = || if rng.gen_bool(0.8) { Some(rng.gen_range(1..=300)) } else { None };

    MatchPlayerRecord {
        match_id,
        player_slot,
        account_id,
        hero_id,
        item_0: item(),
        item_1: item(),
        item_2: item(),
        item_3: item(),
        item_4: item(),
        item_5: item(),
        backpack_0: item(),
        backpack_1: item(),
        backpack_2: item(),
        item_neutral: if rng.gen_bool(0.6) { Some(rng.gen_range(300..=350)) } else { None },
        kills,
        deaths,
        assists,
        kda,
        gold,
        gold_per_min: gpm,
        xp_per_min: xpm,
        gold_spent,
        net_worth,
        hero_damage,
        tower_damage,
        hero_healing,
        level,
        last_hits,
        denies,
        is_radiant,
        win: radiant_win == is_radiant,
        aghanims_scepter: rng.gen_bool(0.15),
        aghanims_shard: rng.gen_bool(0.25),
        rank,
    }
}
