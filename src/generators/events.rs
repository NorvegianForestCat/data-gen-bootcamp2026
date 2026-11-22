//! In-game event document generation for MongoDB.
//!
//! This module provides functions to generate synthetic BSON documents
//! representing various in-game events such as creep score updates,
//! gold acquisitions, hero kills, and tower destructions.

use bson::{doc, oid::ObjectId, DateTime};

/// Generates a Creep Score (CS) event document, typically used for Roshan kills.
///
/// # Arguments
///
/// * `match_id` - The unique identifier of the match.
/// * `account_id` - The Steam account ID of the player who triggered the event.
/// * `time` - The time in seconds when the event occurred.
/// * `npc_id` - The identifier of the NPC involved (e.g., 133 for Roshan).
///
/// # Returns
///
/// A `bson::Document` representing the CS event.
pub fn generate_cs_event(match_id: i64, account_id: i64, time: i32, npc_id: i32) -> bson::Document {
    doc! {
        "_id": ObjectId::new(),
        "match_id": match_id,
        "steam_account_id": account_id,
        "time": time,
        "attacker": account_id,
        "npcId": npc_id,
        "isFromIllusion": false,
        "isCreep": false,
        "isNeutral": true,
        "isAncient": true,
        "gold": 200,
        "xp": 300,
        "byAbility": null,
        "byItem": null,
        "created_at": DateTime::now(),
    }
}

/// Generates a gold acquisition event document.
///
/// # Arguments
///
/// * `match_id` - The unique identifier of the match.
/// * `account_id` - The Steam account ID of the player receiving the gold.
/// * `time` - The time in seconds when the gold was acquired.
/// * `amount` - The amount of gold received.
/// * `reason` - The reason for the gold acquisition (e.g., "ROSHAN").
/// * `npc_id` - The identifier of the NPC involved, if applicable.
///
/// # Returns
///
/// A `bson::Document` representing the gold event.
pub fn generate_gold_event(match_id: i64, account_id: i64, time: i32, amount: i32, reason: &str, npc_id: i32) -> bson::Document {
    doc! {
        "_id": ObjectId::new(),
        "match_id": match_id,
        "steam_account_id": account_id,
        "time": time,
        "amount": amount,
        "reason": reason,
        "npcId": npc_id,
        "created_at": DateTime::now(),
    }
}

/// Generates a hero kill event document.
///
/// # Arguments
///
/// * `match_id` - The unique identifier of the match.
/// * `attacker_id` - The Steam account ID of the player securing the kill.
/// * `time` - The time in seconds when the kill occurred.
/// * `gold` - The amount of gold awarded for the kill.
/// * `xp` - The amount of experience awarded for the kill.
/// * `target_id` - The Steam account ID of the killed player.
///
/// # Returns
///
/// A `bson::Document` representing the kill event.
pub fn generate_kill_event(match_id: i64, attacker_id: i64, time: i32, gold: i32, xp: i32, target_id: i64) -> bson::Document {
    doc! {
        "_id": ObjectId::new(),
        "match_id": match_id,
        "steam_account_id": attacker_id,
        "time": time,
        "gold": gold,
        "xp": xp,
        "isSolo": true,
        "isGank": false,
        "isSmoke": false,
        "isTpRecently": false,
        "isRuneEffected": false,
        "attacker": attacker_id,
        "target": target_id,
        "byAbility": 0,
        "byItem": null,
        "assist": null,
        "isInvisible": false,
        "isFromIllusion": false,
        "created_at": DateTime::now(),
    }
}

/// Generates a tower destruction event document.
///
/// # Arguments
///
/// * `match_id` - The unique identifier of the match.
/// * `time` - The time in seconds when the tower was destroyed.
/// * `radiant_mask` - Bitmask representing the state of Radiant towers.
/// * `dire_mask` - Bitmask representing the state of Dire towers.
///
/// # Returns
///
/// A `bson::Document` representing the tower death event.
pub fn generate_tower_death_event(match_id: i64, time: i32, radiant_mask: i32, dire_mask: i32) -> bson::Document {
    doc! {
        "_id": ObjectId::new(),
        "match_id": match_id,
        "time": time,
        "radiant": radiant_mask,
        "dire": dire_mask,
        "created_at": DateTime::now(),
    }
}
