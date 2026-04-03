use hecs::World;
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
    pub turn_count: u32,
}

impl App {
    pub fn new() -> Self {
        Self::new_random()
    }

    pub fn new_random() -> Self {
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
            turn_count: 0,
        };
        app.generate_level(None);
        app
    }

    fn load_content() -> Content {
        let path = std::path::Path::new("content.json");
        if path.exists() {
            if let Ok(json) = std::fs::read_to_string(path) {
                if let Ok(content) = serde_json::from_str(&json) {
                    return content;
                }
            }
        }
        Content::default()
    }
}
