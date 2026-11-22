CREATE TABLE IF NOT EXISTS heroes (
    hero_id INTEGER PRIMARY KEY,
    name VARCHAR NOT NULL,
    localized_name VARCHAR NOT NULL,
    primary_attr VARCHAR NOT NULL,
    attack_type VARCHAR NOT NULL,
    roles TEXT[] NOT NULL
);

CREATE TABLE IF NOT EXISTS npcs (
    npc_id INTEGER PRIMARY KEY,
    name VARCHAR NOT NULL,
    status_health DOUBLE PRECISION,
    status_mana DOUBLE PRECISION,
    movement_speed DOUBLE PRECISION,
    attack_range DOUBLE PRECISION,
    attack_damage_min DOUBLE PRECISION,
    attack_damage_max DOUBLE PRECISION,
    attack_rate DOUBLE PRECISION,
    projectile_speed DOUBLE PRECISION,
    team_name VARCHAR,
    is_neutral_unit BOOLEAN,
    is_ancient BOOLEAN,
    can_be_dominated BOOLEAN,
    has_inventory BOOLEAN,
    level DOUBLE PRECISION
);

CREATE TABLE IF NOT EXISTS players (
    account_id BIGINT PRIMARY KEY,
    personaname VARCHAR NOT NULL,
    loaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS matches (
    match_id BIGINT PRIMARY KEY,
    match_seq_num BIGINT NOT NULL,
    start_time BIGINT NOT NULL,
    duration INTEGER NOT NULL,
    game_mode INTEGER NOT NULL,
    lobby_type INTEGER NOT NULL,
    radiant_win BOOLEAN NOT NULL,
    radiant_score INTEGER NOT NULL,
    dire_score INTEGER NOT NULL,
    tower_status_radiant INTEGER NOT NULL,
    tower_status_dire INTEGER NOT NULL,
    barracks_status_radiant INTEGER NOT NULL,
    barracks_status_dire INTEGER NOT NULL,
    cluster INTEGER NOT NULL,
    first_blood_time INTEGER NOT NULL,
    human_players INTEGER NOT NULL,
    loaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    stratz_parsed_at TIMESTAMPTZ,
    discover_run_id BIGINT,
    enrich_run_id BIGINT,
    enrichment_attempts INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS match_players (
    match_id BIGINT NOT NULL REFERENCES matches(match_id),
    player_slot INTEGER NOT NULL,
    account_id BIGINT NOT NULL,
    hero_id INTEGER NOT NULL,
    item_0 INTEGER,
    item_1 INTEGER,
    item_2 INTEGER,
    item_3 INTEGER,
    item_4 INTEGER,
    item_5 INTEGER,
    backpack_0 INTEGER,
    backpack_1 INTEGER,
    backpack_2 INTEGER,
    item_neutral INTEGER,
    kills INTEGER NOT NULL,
    deaths INTEGER NOT NULL,
    assists INTEGER NOT NULL,
    kda DOUBLE PRECISION NOT NULL,
    gold INTEGER NOT NULL,
    gold_per_min INTEGER NOT NULL,
    xp_per_min INTEGER NOT NULL,
    gold_spent INTEGER NOT NULL,
    net_worth INTEGER NOT NULL,
    hero_damage INTEGER NOT NULL,
    tower_damage INTEGER NOT NULL,
    hero_healing INTEGER NOT NULL,
    level INTEGER NOT NULL,
    last_hits INTEGER NOT NULL,
    denies INTEGER NOT NULL,
    is_radiant BOOLEAN NOT NULL,
    win BOOLEAN NOT NULL,
    aghanims_scepter BOOLEAN NOT NULL,
    aghanims_shard BOOLEAN NOT NULL,
    rank INTEGER NOT NULL,
    loaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (match_id, player_slot)
);