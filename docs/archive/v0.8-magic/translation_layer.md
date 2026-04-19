# Translation Layer

To implement the translation layer in Rust, you should use **Serde** for parsing the content file (JSON at runtime; YAML notation is used in design docs for readability only) into an "Intermediate Representation" (IR) and then "bake" that IR into a data-driven ECS structure.

This approach keeps the **Content Files** flexible while the **ECS Components** remain highly performant, type-safe, and cache-friendly.

## 1. The Intermediate Representation (IR)

Define Rust structs that mirror your content file schema exactly. This allows Serde to handle the heavy lifting of deserialization.

Note: because `type` is a reserved keyword in Rust, fields named `type` in the content file require `#[serde(rename = "type")]`.

```rust
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RawSpell {
    pub title: String,
    pub description: String,
    pub mana_cost: RawManaCost,
    pub targeting: RawTargetSpec,
    pub effects: Vec<RawSpellEffect>,
}

#[derive(Deserialize, Debug)]
pub struct RawManaCost {
    pub orange: u32,
    pub purple: u32,
}

#[derive(Deserialize, Debug)]
pub struct RawTargetSpec {
    pub range: Option<u32>, // ignored when selection == "self"
    pub selection: String,  // "entity" | "self" | "location"
}

#[derive(Deserialize, Debug)]
pub struct RawSpellEffect {
    #[serde(rename = "type")]
    pub effect_type: String,            // e.g., "DealDamage"
    pub shape: String,                  // "point" | "circle"
    pub radius: Option<u32>,
    pub application_save: Option<String>, // e.g., "DEX" — rolled once at cast time to resist application
    pub damage_type: Option<String>,
    pub status: Option<RawStatusEffect>,
    pub magnitude: Option<String>,      // dice string e.g. "1d6+2" or flat "50"
}

#[derive(Deserialize, Debug, Clone)]
pub struct RawStatusEffect {
    #[serde(rename = "type")]
    pub status_type: String,
    pub duration: Option<u32>,
    pub magnitude: Option<String>,
    pub recovery_save: Option<String>, // e.g., "WIS" — rolled each turn to remove the status early
}
```

## 2. The Baked ECS Components

Once the data is loaded, you translate it into a "Baked" format. This involves parsing strings into enums and dice strings into typed structs.

```rust
// Baked TargetSpec
pub enum TargetSelection {
    Entity,
    SelfCast,  // "self" is a reserved word in Rust
    Location,
}

pub struct TargetSpec {
    pub range: Option<u32>, // None or ignored when selection is SelfCast
    pub selection: TargetSelection,
}

// Baked Spell component
pub struct Spell {
    pub title: String,
    pub description: String,
    pub mana_cost: ManaCost,
    pub level: u32,           // derived: orange + purple
    pub targeting: TargetSpec,
    pub instructions: Vec<EffectInstruction>,
}

// Baked effect
pub struct EffectInstruction {
    pub opcode: EffectOpCode,         // Enum: DealDamage, GrantStatus, etc.
    pub shape: EffectShape,           // Enum: Point, Circle
    pub radius: Option<u32>,          // Only used when shape is Circle
    pub application_save: Option<Attribute>, // resist initial application; see effects.md
    pub magnitude: Option<Dice>,      // None for effects like GrantStatus with no top-level magnitude
    pub metadata: EffectMetadata,     // Enum/Union for specific types
}

pub struct Dice {
    pub count: u32,
    pub sides: u32,
    pub bonus: i32,
}
// A plain integer like "50" bakes to Dice { count: 0, sides: 0, bonus: 50 }.
```

## 3. The Translation Layer (Factory)

The translation layer acts as a bridge, converting `RawSpell` into the ECS-ready `Spell` component.

```rust
impl RawSpell {
    pub fn bake(&self) -> Spell {
        let level = self.mana_cost.orange + self.mana_cost.purple;
        assert!(
            self.mana_cost.orange == 0 || self.mana_cost.purple == 0,
            "Spell '{}': mixed orange+purple cost is invalid", self.title
        );
        assert!(level >= 1, "Spell '{}': level-0 (free) spells are invalid", self.title);

        let instructions = self.effects.iter().map(|e| {
            EffectInstruction {
                opcode: parse_opcode(&e.effect_type),
                shape: parse_shape(&e.shape),
                radius: e.radius,
                application_save: e.application_save.as_deref().map(parse_attribute),
                magnitude: e.magnitude.as_deref().map(parse_dice_string),
                metadata: match e.effect_type.as_str() {
                    "DealDamage" => EffectMetadata::Damage(
                        parse_damage_type(e.damage_type.as_deref().unwrap())
                    ),
                    "GrantStatus" => EffectMetadata::Status(
                        e.status.clone().unwrap()
                    ),
                    _ => EffectMetadata::None,
                }
            }
        }).collect();

        Spell {
            title: self.title.clone(),
            description: self.description.clone(),
            mana_cost: ManaCost {
                orange: self.mana_cost.orange,
                purple: self.mana_cost.purple,
            },
            level,
            targeting: TargetSpec {
                range: self.targeting.range,
                selection: parse_selection(&self.targeting.selection),
            },
            instructions,
        }
    }
}
```

`parse_dice_string` must handle both the full dice form (`2d6+3`) and the flat integer form (`50`):

```rust
pub fn parse_dice_string(s: &str) -> Dice {
    // Try full form: NdS[+/-B]
    if let Some(caps) = DICE_REGEX.captures(s) { ... }
    // Fall back to flat integer
    else if let Ok(flat) = s.parse::<i32>() {
        Dice { count: 0, sides: 0, bonus: flat }
    } else {
        panic!("Invalid dice string: {}", s)
    }
}
```

## 4. ECS Implementation Strategy

In a Rust ECS (like **hecs**), your execution logic becomes a simple, monolithic system that iterates over these instructions.

### The Execution System

When a cast is initiated:

1. **Mana System**: Checks the `ManaPool` component against `Spell.mana_cost`.
2. **Targeting System**: Prompts the player based on `Spell.targeting` (`TargetSpec`).
3. **Execution Loop**:
    * Iterates through the `instructions` vector.
    * Resolves affected entities using each instruction's `shape` and `radius` relative to the chosen origin.
    * For each `DealDamage` opcode, calculates **Save DC** ($10 + \text{level} + \text{CHA mod}$).
    * Rolls the `magnitude` dice and applies the result.
4. **Cleanup**: Consumes a turn.

### Why this works in Rust

* **Memory Safety**: Using enums with data (sum types) ensures you cannot have a `DealDamage` instruction without a `DamageType`.
* **Performance**: Vectors of instructions are contiguous in memory, making the "Apply Effects" step extremely fast during the ECS system pass.
* **Modularity**: You can add a new effect to `effects.md` by simply adding a new enum variant in Rust and one line in the `bake()` factory.

This setup satisfies your requirement for a data-driven implementation while allowing designers to stay in the "functional land" of simple content files.
