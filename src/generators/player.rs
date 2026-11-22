use rand::Rng;

#[derive(Clone)]
pub struct PlayerRecord {
    pub account_id: i64,
    pub personaname: String,
}

#[derive(Clone)]
pub struct MatchPlayerRecord {
    pub match_id: i64,
    pub player_slot: i32,
    pub account_id: i64,
    pub hero_id: i32,
    pub item_0: Option<i32>,
    pub item_1: Option<i32>,
    pub item_2: Option<i32>,
    pub item_3: Option<i32>,
    pub item_4: Option<i32>,
    pub item_5: Option<i32>,
    pub backpack_0: Option<i32>,
    pub backpack_1: Option<i32>,
    pub backpack_2: Option<i32>,
    pub item_neutral: Option<i32>,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub kda: f32,
    pub gold: i32,
    pub gold_per_min: i32,
    pub xp_per_min: i32,
    pub gold_spent: i32,
    pub net_worth: i32,
    pub hero_damage: i32,
    pub tower_damage: i32,
    pub hero_healing: i32,
    pub level: i32,
    pub last_hits: i32,
    pub denies: i32,
    pub is_radiant: bool,
    pub win: bool,
    pub aghanims_scepter: bool,
    pub aghanims_shard: bool,
    pub rank: i32,
}

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