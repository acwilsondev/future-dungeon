use ratatui::prelude::Color;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertState {
    Sleeping,
    Curious { x: u16, y: u16 },
    Aggressive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hearing {
    pub range: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Noise {
    pub amount: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BossPhaseAction {
    SummonMinions,
    Enrage, // Increases power
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BossPhase {
    pub hp_threshold: i32,
    pub action: BossPhaseAction,
    pub triggered: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Boss {
    pub phases: Vec<BossPhase>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SavingThrowKind {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Head,
    Neck,
    Torso,
    Hands,
    Feet,
    MainHand,
    OffHand,
    AnyHand,
    Ammo,
    LeftFinger,
    RightFinger,
}


#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Equippable {
    pub slot: EquipmentSlot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Equipped {
    pub slot: EquipmentSlot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RenderOrder {
    Map,
    Trap,
    Item,
    Monster,
    Player,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Renderable {
    pub glyph: char,
    pub fg: Color,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterClass {
    Fighter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Class {
    pub class: CharacterClass,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Monster;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Branch {
    Main,
    Gardens,
    Vaults,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownStairs {
    pub destination: (u16, Branch),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpStairs {
    pub destination: (u16, Branch),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Door {
    pub open: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trap {
    pub damage: i32,
    pub revealed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Consumable;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ranged {
    pub range: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RangedWeapon {
    pub range: i32,
    pub range_increment: i32,
    pub damage_bonus: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ammunition;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AreaOfEffect {
    pub radius: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Confusion {
    pub turns: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Poison {
    pub damage: i32,
    pub turns: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Strength {
    pub amount: i32,
    pub turns: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Speed {
    pub turns: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Potion {
    pub heal_amount: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponWeight {
    Light,
    Medium,
    Heavy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Weapon {
    pub power_bonus: i32,
    pub weight: WeaponWeight,
    pub damage_n_dice: i32,
    pub damage_die_type: i32,
    pub two_handed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Armor {
    pub defense_bonus: i32,
    pub max_dex_bonus: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InBackpack {
    pub owner: hecs::Entity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attributes {
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
}

impl Attributes {
    pub fn get_modifier(score: i32) -> i32 {
        (score - 10) / 2
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FactionKind {
    Player,
    Orcs,
    Goblins,
    Animals,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Faction(pub FactionKind);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Personality {
    Brave,
    Cowardly, // Flees at low HP
    Tactical, // Stays at range
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AIPersonality(pub Personality);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Viewshed {
    pub visible_tiles: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LightSource {
    pub range: i32,
    pub base_range: i32,
    pub color: (u8, u8, u8),
    pub remaining_turns: Option<i32>,
    pub flicker: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wisp;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Name(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObfuscatedName(pub String);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cursed;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlchemyStation;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HolyAltar;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResetShrine;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Experience {
    pub level: i32,
    pub xp: i32,
    pub next_level_xp: i32,
    pub xp_reward: i32, // How much XP this entity gives when killed
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Perk {
    Toughness, // +Max HP
    EagleEye,  // +FOV
    Strong,    // +Power
    ThickSkin, // +Defense
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Perks {
    pub traits: Vec<Perk>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LastHitByPlayer;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gold {
    pub amount: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Merchant;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemValue {
    pub price: i32,
}
