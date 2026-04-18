# Magic Update Implementation Plan

This document outlines the architectural and implementation steps required to integrate the Magic Update into the `rustlike` codebase as defined in `docs/design/mechanics/magic.md`.

## 1. Data Models (Components & Types)

### New Enums in `components.rs`

- `ManaColor`: `Orange`, `Purple`
- `TargetingStrategy`: `SingleObject`, `FloorSpot`, `InventoryItem`.
- `EffectType`: `Damage`, `Heal`, `Teleport`, etc.

### New Components in `components.rs`

- `ManaPool`:

    ```rust
    pub struct ManaPool {
        pub current_orange: i32,
        pub max_orange: i32,
        pub current_purple: i32,
        pub max_purple: i32,
    }
    ```

- `ManaRecoveryClock`:

    ```rust
    pub struct ManaRecoveryClock {
        pub timer: i32,      // 0 to 10
        pub wait_timer: i32, // 5 to 0
    }
    ```

- `Spell`:

    ```rust
    pub struct Spell {
        pub title: String,
        pub orange_cost: i32,
        pub purple_cost: i32,
        pub targeting: TargetingStrategy,
        pub effects: Vec<SpellEffect>,
    }
    ```

- `KnownSpells`: `pub struct KnownSpells(pub Vec<hecs::Entity>);`
- `Tome`:

    ```rust
    pub struct Tome {
        pub spell_id: String, // Reference to raw content
        pub origin: ManaColor,
    }
    ```

- `Shrine`:

    ```rust
    pub struct Shrine {
        pub color: ManaColor,
        pub used: bool,
    }
    ```

## 2. Core Mechanics Integration

### Mana Recovery (`app/turn_tick.rs`)

Add `handle_mana_recovery` to `on_turn_tick`:

1. If mana was spent this turn, reset `wait_timer` to 5 and `timer` to 0.
2. If `wait_timer > 0`, decrement it.
3. If `wait_timer == 0` and total current mana < total max mana:
    - Increment `timer`.
    - If `timer == 10`:
        - Reset `timer` to 0.
        - Identify missing colors.
        - Grant 1 mana of a missing color (randomly if multiple colors have same missing amount).

### Spell Casting Flow (`actions.rs`)

Implement `Action::CastSpell(Entity)`:

1. **Mana Check**: Compare `ManaPool` against `Spell` costs.
2. **Choose Targets**: Transition to `RunState::ShowTargeting` with specific qualifiers.
3. **Pay Mana**: Deduct from `ManaPool`.
4. **Apply Effects**: procedural logic based on `SpellEffectType`.
5. **Cleanup**: Reset `ManaRecoveryClock`.

### Studying Tomes & Shrines (`actions.rs`)

- `Action::StudyTome(Entity)`: CHA check vs `5 + 2 * SpellLevel`.
- `Action::Meditate(Entity)`: CHA check vs `10 + TotalMaxMana`.

## 3. UI Implementation (`renderer.rs`)

### Sidebar Enhancements

- Within `draw_sidebar`, implement `draw_mana_bar`:
  - Display 1-5 asterisks `*`.
  - Colors: `Color::Orange`, `Color::Magenta` (Purple), `Color::DarkGray` (Spent).
  - Draw the "overline" recovery clock using a thin bar (e.g., using `ratatui::widgets::Canvas` or custom symbols).

### Abilities Screen

- Implement `render_abilities_menu`:
  - Accessible via `Action::OpenAbilities` (mapped to `a`).
  - List spells from `KnownSpells`.
  - Display mana costs and descriptions.

## 4. Systems & Input Updates

### `input.rs`

- Map `KeyCode::Char('a')` to `Action::OpenAbilities` in `RunState::AwaitingInput`.
- Map `KeyCode::Char('a')` to `Action::CloseMenu` in `RunState::ShowAbilities`.

### `app/state.rs`

- Add `RunState::ShowAbilities`.

### `spawner.rs` & `content.rs`

- Update `RawItem` to include optional `Tome` data.
- Add `spawn_tome` and `spawn_shrine` functions.
- Update `add_item_components` to handle Tomes.

## 5. Identification System

- Integrate `Tome` into `identified_items` HashSet in `App`.
- Ensure `get_item_name` handles identified vs unidentified Tomes (e.g., "Unidentified Nihil Tome" vs "Tome of Fireball").

## 6. Execution Roadmap

1. **Phase 1**: Add components and basic mana pool to Player spawn.
2. **Phase 2**: Implement turn-based mana recovery logic.
3. **Phase 3**: Implement the sidebar UI for mana visualization.
4. **Phase 4**: Implement Tome item spawning and the "Study" action.
5. **Phase 5**: Implement the Abilities menu and the 5-step spell casting workflow.
6. **Phase 6**: Add Shrines and mana pool expansion logic.
