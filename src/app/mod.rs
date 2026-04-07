use hecs::World;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::components::Branch;
use crate::content::Content;
use crate::map::Map;

mod actions;
mod actions_alchemy;
mod actions_item;
mod actions_levelup;
mod actions_shop;
mod combat;
mod helpers;
mod items_equip;
mod items_shop;
mod items_use_basic;
mod items_use_ranged;
mod level_gen;
mod level_gen_helpers;
mod level_transition;
mod monster;
mod monster_ai_calc;
mod monster_ai_execute;
mod monster_boss;
mod monster_perception;
mod player_move;
mod serialization;
mod snapshot;
mod state;
mod turn_tick;
mod visual_effects;
mod world_update;

pub use snapshot::*;
pub use state::*;

#[derive(Serialize, Deserialize)]
pub struct App {
    #[serde(skip)]
    pub exit: bool,
    #[serde(skip)]
    pub death: bool,
    #[serde(skip, default = "World::new")]
    pub world: World,
    pub map: Map,
    pub entities: Vec<EntitySnapshot>,
    #[serde(
        serialize_with = "serialize_levels",
        deserialize_with = "deserialize_levels"
    )]
    pub levels: HashMap<(u16, Branch), LevelData>,
    pub log: Vec<String>,
    pub dungeon_level: u16,
    pub current_branch: Branch,
    #[serde(skip, default = "default_runstate")]
    pub state: RunState,
    #[serde(skip)]
    pub inventory_cursor: usize,
    #[serde(skip)]
    pub targeting_cursor: (u16, u16),
    #[serde(skip)]
    pub targeting_item: Option<hecs::Entity>,
    #[serde(skip)]
    pub speed_toggle: bool,
    #[serde(skip)]
    pub level_up_cursor: usize,
    #[serde(skip)]
    pub shop_cursor: usize,
    #[serde(skip)]
    pub active_merchant: Option<hecs::Entity>,
    #[serde(skip)]
    pub shop_mode: usize, // 0 = Buy, 1 = Sell
    #[serde(skip)]
    pub effects: Vec<VisualEffect>,
    #[serde(skip)]
    pub log_cursor: usize,
    pub encountered_monsters: std::collections::HashSet<String>,
    pub identified_items: std::collections::HashSet<String>,
    #[serde(skip)]
    pub bestiary_cursor: usize,
    pub content: Content,
    #[serde(skip)]
    pub fps: f32,
    pub escaping: bool,
    pub monsters_killed: u32,
    #[serde(skip)]
    pub alchemy_selection: Vec<hecs::Entity>,
    #[serde(skip, default = "ChaCha8Rng::from_entropy")]
    pub rng: ChaCha8Rng,
    pub seed: u64,
    pub turn_count: u32,
}

impl App {
    pub fn new() -> Self {
        Self::new_random()
    }

    pub fn new_random() -> Self {
        let seed = rand::random::<u64>();
        let mut app = Self {
            exit: false,
            death: false,
            world: World::new(),
            map: Map::new(80, 50),
            entities: Vec::new(),
            levels: HashMap::new(),
            log: vec!["Welcome to RustLike!".to_string()],
            dungeon_level: 1,
            current_branch: Branch::Main,
            state: RunState::AwaitingInput,
            inventory_cursor: 0,
            targeting_cursor: (0, 0),
            targeting_item: None,
            speed_toggle: true,
            level_up_cursor: 0,
            shop_cursor: 0,
            active_merchant: None,
            shop_mode: 0,
            effects: Vec::new(),
            log_cursor: 0,
            encountered_monsters: std::collections::HashSet::new(),
            identified_items: std::collections::HashSet::new(),
            bestiary_cursor: 0,
            content: Self::load_content(),
            fps: 0.0,
            escaping: false,
            monsters_killed: 0,
            alchemy_selection: Vec::new(),
            rng: ChaCha8Rng::seed_from_u64(seed),
            seed,
            turn_count: 0,
        };
        app.generate_level(Vec::new());
        app
    }

    pub fn new_test(seed: u64) -> Self {
        let mut app = Self {
            exit: false,
            death: false,
            world: World::new(),
            map: Map::new(80, 50),
            entities: Vec::new(),
            levels: HashMap::new(),
            log: vec!["Welcome to RustLike!".to_string()],
            dungeon_level: 1,
            current_branch: Branch::Main,
            state: RunState::AwaitingInput,
            inventory_cursor: 0,
            targeting_cursor: (0, 0),
            targeting_item: None,
            speed_toggle: true,
            level_up_cursor: 0,
            shop_cursor: 0,
            active_merchant: None,
            shop_mode: 0,
            effects: Vec::new(),
            log_cursor: 0,
            encountered_monsters: std::collections::HashSet::new(),
            identified_items: std::collections::HashSet::new(),
            bestiary_cursor: 0,
            content: Self::load_content(),
            fps: 0.0,
            escaping: false,
            monsters_killed: 0,
            alchemy_selection: Vec::new(),
            rng: ChaCha8Rng::seed_from_u64(seed),
            seed,
            turn_count: 0,
        };
        app.generate_level(Vec::new());
        app
    }

    fn load_content() -> Content {
        let path = std::path::Path::new("content.json");
        if path.exists() {
            match std::fs::read_to_string(path) {
                Ok(json) => {
                    match serde_json::from_str::<Content>(&json) {
                        Ok(content) => return content,
                        Err(e) => log::error!("Failed to parse content.json: {}", e),
                    }
                }
                Err(e) => log::error!("Failed to read content.json: {}", e),
            }
        } else {
            log::warn!("content.json not found at {:?}", path);
        }
        Content::default()
    }
}

pub fn serialize_levels<S>(
    levels: &HashMap<(u16, Branch), LevelData>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let vec: Vec<((u16, Branch), LevelData)> =
        levels.iter().map(|(k, v)| (*k, v.clone())).collect();
    vec.serialize(serializer)
}

pub fn deserialize_levels<'de, D>(
    deserializer: D,
) -> Result<HashMap<(u16, Branch), LevelData>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let vec: Vec<((u16, Branch), LevelData)> = Vec::deserialize(deserializer)?;
    let mut map = HashMap::new();
    for (k, mut v) in vec {
        v.map.reinitialize_skipped_fields();
        map.insert(k, v);
    }
    Ok(map)
}
