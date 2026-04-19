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
pub enum Biome {
    Dungeon,
    Crypt,
    Caves,
    Temple,
    Hell,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FloorModifier {
    None,
    Dark,
    Bright,
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
        (score - 10).div_euclid(2)
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
    Undead,
    Demons,
    Temple,
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
pub struct Levitation;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Regeneration;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Merchant;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemValue {
    pub price: i32,
}

// ======== Magic / Spells ========

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ManaColor {
    Orange,
    Purple,
}

impl ManaColor {
    #[allow(dead_code)]
    pub fn display_color(&self) -> Color {
        match self {
            ManaColor::Orange => Color::Rgb(255, 165, 0),
            ManaColor::Purple => Color::Rgb(160, 90, 200),
        }
    }

    pub fn order_name(&self) -> &'static str {
        match self {
            ManaColor::Orange => "Solari",
            ManaColor::Purple => "Nihil",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManaCost {
    pub orange: u32,
    pub purple: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManaPool {
    pub current_orange: u32,
    pub max_orange: u32,
    pub current_purple: u32,
    pub max_purple: u32,
}

impl ManaPool {
    pub const CAP: u32 = 5;

    pub fn total_max(&self) -> u32 {
        self.max_orange + self.max_purple
    }
    pub fn total_current(&self) -> u32 {
        self.current_orange + self.current_purple
    }
    pub fn has_mana_for(&self, cost: &ManaCost) -> bool {
        self.current_orange >= cost.orange && self.current_purple >= cost.purple
    }
    pub fn pay(&mut self, cost: &ManaCost) {
        self.current_orange = self.current_orange.saturating_sub(cost.orange);
        self.current_purple = self.current_purple.saturating_sub(cost.purple);
    }
    /// Returns true if max could be increased.
    pub fn increase_max(&mut self, color: ManaColor) -> bool {
        if self.total_max() >= Self::CAP {
            return false;
        }
        match color {
            ManaColor::Orange => {
                self.max_orange += 1;
                self.current_orange += 1;
            }
            ManaColor::Purple => {
                self.max_purple += 1;
                self.current_purple += 1;
            }
        }
        true
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Attribute {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

impl Attribute {
    pub fn to_saving_throw_kind(self) -> SavingThrowKind {
        match self {
            Attribute::Strength => SavingThrowKind::Strength,
            Attribute::Dexterity => SavingThrowKind::Dexterity,
            Attribute::Constitution => SavingThrowKind::Constitution,
            Attribute::Intelligence => SavingThrowKind::Intelligence,
            Attribute::Wisdom => SavingThrowKind::Wisdom,
            Attribute::Charisma => SavingThrowKind::Charisma,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetSelection {
    Entity,
    SelfCast,
    Location,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetSpec {
    pub range: Option<u32>,
    pub selection: TargetSelection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageType {
    Fire,
    Poison,
    Bludgeoning,
    Slashing,
    Piercing,
    Necrotic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dice {
    pub count: u32,
    pub sides: u32,
    pub bonus: i32,
}

impl Dice {
    pub fn flat(v: i32) -> Self {
        Self {
            count: 0,
            sides: 0,
            bonus: v,
        }
    }

    pub fn roll<R: rand::Rng>(&self, rng: &mut R) -> i32 {
        let mut total = self.bonus;
        if self.sides > 0 {
            for _ in 0..self.count {
                total += rng.random_range(1..=self.sides as i32);
            }
        }
        total
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BakedStatusEffect {
    pub status_type: String,
    pub duration: Option<u32>,
    pub magnitude: Option<Dice>,
    pub recovery_save: Option<Attribute>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectMetadata {
    None,
    Damage(DamageType),
    Status(BakedStatusEffect),
    RemoveStatus(String),
    Vector { x: i32, y: i32 },
    CreateEntity(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectOpCode {
    DealDamage,
    GrantStatus,
    RemoveStatus,
    Heal,
    Push,
    Teleport,
    CreateEntity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectShape {
    Point,
    Circle,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectInstruction {
    pub opcode: EffectOpCode,
    pub shape: EffectShape,
    pub radius: Option<u32>,
    pub application_save: Option<Attribute>,
    pub magnitude: Option<Dice>,
    pub metadata: EffectMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spell {
    pub title: String,
    pub description: String,
    pub mana_cost: ManaCost,
    pub level: u32,
    pub targeting: TargetSpec,
    pub instructions: Vec<EffectInstruction>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spellbook {
    pub spells: Vec<Spell>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManaDrought {
    pub duration: u32,
}

/// Generic "slowed by terrain" status. Used by spell effects (e.g., Mired).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mired {
    pub magnitude: i32,
    pub duration: u32,
    pub recovery_save: Option<Attribute>,
}

/// Generic armor buff from magic (e.g., Armored status).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Armored {
    pub magnitude: i32,
    pub duration: u32,
    pub recovery_save: Option<Attribute>,
}

/// Shrine entity. `tried` is true once this shrine has been attempted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shrine {
    pub color: ManaColor,
    pub tried: bool,
}

/// Item component indicating this item is a Tome teaching a specific spell.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tome {
    pub spell_name: String,
    pub color: ManaColor,
    pub level: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_modifier_floor_division() {
        assert_eq!(Attributes::get_modifier(10), 0);
        assert_eq!(Attributes::get_modifier(11), 0);
        assert_eq!(Attributes::get_modifier(12), 1);
        assert_eq!(Attributes::get_modifier(9), -1); // was 0 with truncating division
        assert_eq!(Attributes::get_modifier(8), -1);
        assert_eq!(Attributes::get_modifier(7), -2);
        assert_eq!(Attributes::get_modifier(1), -5);
    }
}
