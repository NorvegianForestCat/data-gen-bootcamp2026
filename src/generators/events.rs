use bson::{doc, oid::ObjectId, DateTime};

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