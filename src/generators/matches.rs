use rand::Rng;
use rand::seq::SliceRandom;
use chrono::Utc;
use bson::Document;
use crate::generators::player::{generate_player_record, generate_match_player_record, PlayerRecord, MatchPlayerRecord};
use crate::generators::events::{generate_cs_event, generate_gold_event, generate_kill_event, generate_tower_death_event};

pub struct MatchRecord {
    pub match_id: i64,
    pub match_seq_num: i64,
    pub start_time: i32,
    pub duration: i32,
    pub game_mode: i32,
    pub lobby_type: i32,
    pub radiant_win: bool,
    pub radiant_score: i32,
    pub dire_score: i32,
    pub tower_status_radiant: i32,
    pub tower_status_dire: i32,
    pub barracks_status_radiant: i32,
    pub barracks_status_dire: i32,
    pub cluster: i32,
    pub first_blood_time: i32,
    pub human_players: i32,
}

pub struct GeneratedMatch {
    pub match_record: MatchRecord,
    pub player_records: Vec<MatchPlayerRecord>,
    pub unique_players: Vec<PlayerRecord>,
    pub mongo_events: Vec<Document>,
}

pub fn generate_match(rng: &mut impl Rng, match_id: i64, match_seq_num: i64) -> GeneratedMatch {
    let duration: i32 = rng.gen_range(600..3600); // 10-60 минут
    let game_mode: i32 = *[1, 2, 3, 22].choose(rng).unwrap();
    let lobby_type: i32 = *[0, 7].choose(rng).unwrap();
    let radiant_win: bool = rng.gen_bool(0.5);
    let radiant_score: i32 = rng.gen_range(10..60);
    let dire_score: i32 = rng.gen_range(10..60);
    
    let tower_status_radiant: i32 = if radiant_win { rng.gen_range(1000..2047) } else { rng.gen_range(0..500) };
    let tower_status_dire: i32 = if !radiant_win { rng.gen_range(1000..2047) } else { rng.gen_range(0..500) };
    let barracks_status_radiant: i32 = if radiant_win { 63 } else { rng.gen_range(0..30) };
    let barracks_status_dire: i32 = if !radiant_win { 63 } else { rng.gen_range(0..30) };
    
    let cluster: i32 = rng.gen_range(100..200);
    let first_blood_time: i32 = rng.gen_range(30..300);

    let match_record = MatchRecord {
        match_id,
        match_seq_num,
        start_time: Utc::now().timestamp() as i32 - duration,
        duration,
        game_mode,
        lobby_type,
        radiant_win,
        radiant_score,
        dire_score,
        tower_status_radiant,
        tower_status_dire,
        barracks_status_radiant,
        barracks_status_dire,
        cluster,
        first_blood_time,
        human_players: 10,
    };

    let mut player_records = Vec::new();
    let mut unique_players = Vec::new();
    let mut mongo_events = Vec::new();

    let mut account_ids = Vec::new();
    for i in 0..10 {
        let player_slot = if i < 5 { i } else { 128 + (i - 5) };
        let account_id = 100_000_000 + rng.gen_range(0..900_000_000);
        let p = generate_player_record(rng, account_id);
        account_ids.push(p.account_id);
        unique_players.push(p);
        
        let mp = generate_match_player_record(rng, match_id, player_slot, duration, radiant_win, i < 5);
        player_records.push(mp);
    }

    // Генерация событий
    for (i, &acc_id) in account_ids.iter().enumerate() {
        let player_slot = if i < 5 { i } else { 128 + (i - 5) };
        let is_radiant = i < 5;
        
        if rng.gen_bool(0.15) {
            let roshan_time = rng.gen_range(600..(duration - 300)).max(600);
            mongo_events.push(generate_cs_event(match_id, acc_id, roshan_time, 133));
            mongo_events.push(generate_gold_event(match_id, acc_id, roshan_time, 200, "ROSHAN", 133));
        }
        
        if rng.gen_bool(0.4) {
            let kill_time = rng.gen_range(0..duration);
            let target_id = 100_000_000 + rng.gen_range(0..900_000_000);
            mongo_events.push(generate_kill_event(
                match_id, acc_id, kill_time, 
                rng.gen_range(200..500), 
                rng.gen_range(200..500),
                target_id
            ));
        }
    }

    if rng.gen_bool(0.7) {
        let tower_time = rng.gen_range(300..duration);
        mongo_events.push(generate_tower_death_event(match_id, tower_time, if radiant_win { 2047 } else { 0 }, if radiant_win { 0 } else { 2047 }));
    }

    GeneratedMatch {
        match_record,
        player_records,
        unique_players,
        mongo_events,
    }
}