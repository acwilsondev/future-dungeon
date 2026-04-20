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
mod actions_spells;
mod casting;
mod combat;
mod damage;
mod helpers;
mod items_equip;
mod items_shop;
mod items_use_basic;
mod items_use_ranged;
mod level_gen;
mod level_gen_helpers;
mod level_transition;
mod mana_regen;
mod monster;
mod monster_ai_calc;
mod monster_ai_execute;
mod monster_boss;
mod monster_perception;
mod player_move;
mod serialization;
mod shrine;
mod snapshot;
mod state;
mod tome;
mod turn_tick;
mod visual_effects;
mod world_update;

pub use damage::DamageRoute;
pub use snapshot::EntitySnapshot;
pub use state::{default_runstate, MonsterAction, RunState, ShopMode, VisualEffect};

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
    pub shop_mode: ShopMode,
    #[serde(skip)]
    pub effects: Vec<VisualEffect>,
    #[serde(skip)]
    pub log_cursor: usize,
    #[serde(skip)]
    pub alchemy_selection: Vec<hecs::Entity>,
    #[serde(skip)]
    pub spell_cursor: usize,
    #[serde(skip)]
    pub casting_spell: Option<crate::components::Spell>,
    #[serde(skip)]
    pub shrine_entity: Option<hecs::Entity>,
    #[serde(skip)]
    pub study_tome_entity: Option<hecs::Entity>,
    #[serde(skip)]
    pub yes_no_cursor: usize,
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
        let mut app = Self::build(seed, Content::load()?);
        app.init_stars();
        Ok(app)
    }

    fn build(seed: u64, content: Content) -> Self {
        Self {
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
            shop_mode: ShopMode::Buy,
            effects: Vec::new(),
            log_cursor: 0,
            alchemy_selection: Vec::new(),
            spell_cursor: 0,
            casting_spell: None,
            shrine_entity: None,
            study_tome_entity: None,
            yes_no_cursor: 0,
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
            content,
        }
    }
}

#[cfg(test)]
impl App {
    pub fn new_test(seed: u64) -> Self {
        let content = Content::load().expect("content.json must be present for tests");
        let mut app = Self::build(seed, content);
        app.init_stars();
        app
    }
}
