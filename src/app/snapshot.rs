use crate::components::*;
use crate::map::Map;
use serde::{Deserialize, Serialize};

/// A snapshot of an entity for serialization
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EntitySnapshot {
    pub pos: Option<Position>,
    pub render: Renderable,
    pub render_order: RenderOrder,
    pub name: Option<Name>,
    pub stats: Option<CombatStats>,
    #[serde(default)]
    pub potion: Option<Potion>,
    #[serde(default)]
    pub weapon: Option<Weapon>,
    #[serde(default)]
    pub armor: Option<Armor>,
    #[serde(default)]
    pub door: Option<Door>,
    #[serde(default)]
    pub trap: Option<Trap>,
    #[serde(default)]
    pub ranged: Option<Ranged>,
    #[serde(default)]
    pub ranged_weapon: Option<RangedWeapon>,
    #[serde(default)]
    pub aoe: Option<AreaOfEffect>,
    #[serde(default)]
    pub confusion: Option<Confusion>,
    #[serde(default)]
    pub poison: Option<Poison>,
    #[serde(default)]
    pub strength: Option<Strength>,
    #[serde(default)]
    pub speed: Option<Speed>,
    #[serde(default)]
    pub faction: Option<Faction>,
    #[serde(default)]
    pub viewshed: Option<Viewshed>,
    #[serde(default)]
    pub personality: Option<AIPersonality>,
    #[serde(default)]
    pub experience: Option<Experience>,
    #[serde(default)]
    pub perks: Option<Perks>,
    #[serde(default)]
    pub alert_state: Option<AlertState>,
    #[serde(default)]
    pub hearing: Option<Hearing>,
    #[serde(default)]
    pub boss: Option<Boss>,
    #[serde(default)]
    pub light_source: Option<LightSource>,
    #[serde(default)]
    pub gold: Option<Gold>,
    #[serde(default)]
    pub item_value: Option<ItemValue>,
    #[serde(default)]
    pub obfuscated_name: Option<ObfuscatedName>,
    #[serde(default)]
    pub cursed: Option<Cursed>,
    #[serde(default)]
    pub equippable: Option<Equippable>,
    #[serde(default)]
    pub equipped: Option<Equipped>,
    #[serde(default)]
    pub last_hit_by_player: bool,
    #[serde(default)]
    pub is_merchant: bool,
    #[serde(default)]
    pub ammo: bool,
    #[serde(default)]
    pub consumable: bool,
    #[serde(default)]
    pub in_backpack: bool,
    pub is_player: bool,
    pub is_monster: bool,
    #[serde(default)]
    pub is_wisp: bool,
    #[serde(default)]
    pub is_item: bool,
    #[serde(default)]
    pub is_down_stairs: bool,
    #[serde(default)]
    pub is_up_stairs: bool,
    #[serde(default)]
    pub destination: Option<(u16, Branch)>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LevelData {
    pub map: Map,
    pub entities: Vec<EntitySnapshot>,
}
