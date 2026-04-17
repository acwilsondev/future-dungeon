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
mod actions_respec;
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

pub use snapshot::EntitySnapshot;
pub use state::{default_runstate, MonsterAction, RunState, VisualEffect};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Star {
    pub x: f64,
    pub y: f64,
    pub speed: f64,
    pub brightness: u8,
}

#[derive(Serialize, Deserialize)]
pub struct App {
    pub exit: bool,
    pub death: bool,
    #[serde(skip)]
    pub world: World,
    pub map: Map,
    #[serde(skip, default = "default_rng")]
    pub rng: ChaCha8Rng,
    pub entities: Vec<EntitySnapshot>,
    pub levels: HashMap<(u16, Branch), LevelData>,
    pub log: Vec<String>,
    pub dungeon_level: u16,
    pub current_branch: Branch,
    #[serde(skip, default = "default_runstate")]
    pub state: RunState,
    #[serde(skip)]
    pub main_menu_cursor: usize,
    #[serde(skip)]
    pub stars: Vec<Star>,
    #[serde(skip)]
    pub class_selection: usize,
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
    pub respec_points: i32,
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
    #[serde(skip)]
    pub alchemy_selection: Vec<hecs::Entity>,
    #[serde(skip)]
    pub fps: f64,
    #[serde(skip)]
    pub debug_console_buffer: String,
    #[serde(skip)]
    pub god_mode: bool,

    // Persistence/State
    pub identified_items: std::collections::HashSet<String>,
    pub encountered_monsters: std::collections::HashSet<String>,
    pub bestiary_cursor: usize,
    pub monsters_killed: i32,
    pub turn_count: u32,
    pub escaping: bool,
    pub content: Content,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LevelData {
    pub map: Map,
    pub entities: Vec<EntitySnapshot>,
}

fn default_rng() -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(0)
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        Self::new_random()
    }

    pub fn new_random() -> anyhow::Result<Self> {
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
            state: RunState::MainMenu,
            main_menu_cursor: 0,
            stars: Vec::new(),
            class_selection: 0,
            inventory_cursor: 0,
            targeting_cursor: (0, 0),
            targeting_item: None,
            speed_toggle: true,
            level_up_cursor: 0,
            respec_points: 0,
            shop_cursor: 0,
            active_merchant: None,
            shop_mode: 0,
            effects: Vec::new(),
            log_cursor: 0,
            alchemy_selection: Vec::new(),
            fps: 0.0,
            debug_console_buffer: String::new(),
            god_mode: false,
            identified_items: std::collections::HashSet::new(),
            encountered_monsters: std::collections::HashSet::new(),
            bestiary_cursor: 0,
            monsters_killed: 0,
            turn_count: 0,
            escaping: false,
            rng: ChaCha8Rng::seed_from_u64(seed),
            content: Content::load()?,
        };
        app.init_stars();
        Ok(app)
    }
}

#[cfg(test)]
impl App {
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
            state: RunState::MainMenu,
            main_menu_cursor: 0,
            stars: Vec::new(),
            class_selection: 0,
            inventory_cursor: 0,
            targeting_cursor: (0, 0),
            targeting_item: None,
            speed_toggle: true,
            level_up_cursor: 0,
            respec_points: 0,
            shop_cursor: 0,
            active_merchant: None,
            shop_mode: 0,
            effects: Vec::new(),
            log_cursor: 0,
            alchemy_selection: Vec::new(),
            fps: 0.0,
            debug_console_buffer: String::new(),
            god_mode: false,
            identified_items: std::collections::HashSet::new(),
            encountered_monsters: std::collections::HashSet::new(),
            bestiary_cursor: 0,
            monsters_killed: 0,
            turn_count: 0,
            escaping: false,
            rng: ChaCha8Rng::seed_from_u64(seed),
            content: Content::load().expect("content.json must be present for tests"),
        };
        app.init_stars();
        app
    }
}
