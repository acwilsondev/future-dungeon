# Implementation Plan: v0.8-magic

**Status:** Draft
**Epic:** v0.8-magic

This document outlines the technical changes required to implement the magic system as defined in the design documents.

## 1. Components (`src/components.rs`)

### 1.1 Mana System
- **`ManaPool`**: 
    - `current_orange`, `max_orange`: `u32`
    - `current_purple`, `max_purple`: `u32`
    - Capped at 5 total max mana (`max_orange + max_purple <= 5`).

### 1.2 Spellbook
- **`Spellbook`**: 
    - `spells`: `Vec<Entity>` (Entities that hold `Spell` component)
- **`Spell` (Baked)**:
    - `title`: `String`
    - `description`: `String`
    - `mana_cost`: `ManaCost` (struct with orange/purple `u32` fields)
    - `level`: `u32` (derived: `orange + purple`)
    - `targeting`: `TargetSpec` (baked from `RawTargetSpec`)
    - `instructions`: `Vec<EffectInstruction>`
- **`TargetSpec`** (baked):
    - `range`: `Option<u32>` (ignored when selection is `SelfCast`)
    - `selection`: `TargetSelection` enum (`Entity`, `SelfCast`, `Location`)

### 1.3 Effect Instructions
- **`EffectOpCode`**: Enum (`DealDamage`, `GrantStatus`, `RemoveStatus`, `Heal`, `Push`, `Teleport`, `CreateEntity`)
- **`EffectShape`**: Enum (`Point`, `Circle`)
- **`EffectInstruction`**:
    - `opcode`: `EffectOpCode`
    - `shape`: `EffectShape`
    - `radius`: `Option<u32>` (only used when `shape` is `Circle`)
    - `application_save`: `Option<Attribute>` (resist initial application; see `effects.md`)
    - `magnitude`: `Option<Dice>` (None for effects with no top-level roll, e.g. `GrantStatus`)
    - `metadata`: `EffectMetadata` (enum for specific data like `DamageType` or `RawStatusEffect`)

### 1.4 Status Effects
- Add `ManaDrought` component (applied when total current mana across all colors hits zero).
- Add `recovery_save: Option<Attribute>` field to the baked `StatusEffect` struct; rolled each turn to remove the status early.
- Expand status effect components to support new magic-related effects (e.g., `AegisBoost`, `Armored`, `Warped`).

---

## 2. Content System (`src/content.rs`)

### 2.1 Raw Spells
- **`RawSpell`**: Mirrors content file definition (see `translation_layer.md`).
- **`RawSpellEffect`**: Mirrors content file definition.
- Update **`Content`** struct to include `pub spells: Vec<RawSpell>`.
- Spell content uses **JSON format** consistent with the existing `content.json`. Design documents use YAML notation for readability, but the actual file loaded at runtime is JSON.

### 2.2 Translation Layer
- Implement `bake()` method for `RawSpell` to convert it into the ECS-ready `Spell` component.
- Implement dice string parser supporting both full form (`2d6+3`) and flat integer (`50`). See `status_effects.md` for the regex spec.
- Validate at load time: reject spells where both costs are non-zero (mixed-color) or both are zero (level 0).

---

## 3. Game State & Actions

### 3.1 RunState (`src/app/state.rs`)
- Add `RunState::ShowSpells`.
- Add `RunState::SpellTargeting` if standard targeting needs specialization.

### 3.2 Actions (`src/actions.rs` & `src/app/actions.rs`)
- Add `Action::OpenSpells` (mapped to `a` key).
- Implement `handle_spells_input` in `src/app/actions_spells.rs` (new file, consistent with `actions_shop.rs`, `actions_alchemy.rs` pattern).
    - Navigate known spells.
    - Select spell to initiate casting flow.

---

## 4. Systems

### 4.1 Mana Regeneration (`src/app/mana_regen.rs`)
- New system integrated into `on_turn_tick`.
- Logic: if the entity has an active `ManaDrought` status, skip regen. Otherwise grant +1 mana, randomly selecting among the colors with the greatest deficit (max minus current). If deficits are equal, choose randomly.

### 4.2 Spellcasting Flow (`src/app/casting.rs`)
- Implement the 5-step flow:
    1. **Mana Check**: Compare `ManaPool` vs `Spell.mana_cost`.
    2. **Targeting**: Use existing targeting infrastructure but pass `TargetSpec`.
    3. **Pay Mana**: Deduct from `ManaPool`. If `ManaPool` is now empty, apply `ManaDrought(5)` status.
    4. **Apply Effects**: Iterate over `Spell.instructions` and execute opcodes. For each instruction, resolve affected entities using `shape`/`radius` relative to the chosen target origin.
    5. **Cleanup**: End turn, trigger `RunState::MonsterTurn`.

### 4.3 Status Effect System (`src/app/turn_tick.rs`)
- Update `apply_status_effects` to handle new magic statuses.
- Support saving throws for status recovery.

---

## 5. UI & Rendering (`src/renderer.rs`)

### 5.1 Character Pane (Sidebar)
- Implement Mana pip visualization in `draw_sidebar`.
- If `max_orange == 0 && max_purple == 0`, hide the mana section entirely.
- Pip ordering (left to right): Orange Unspent → Purple Unspent → Orange Spent → Purple Spent.
- Colors: Solari (Orange), Nihil (Purple), Spent (Grey/Dulled).

### 5.2 Abilities Screen
- Implement `render_spells` method to show known spells, mana costs, and descriptions.

### 5.3 Targeting UI
- Update `draw_targeting_line` to handle spell-specific shapes.
- During targeting, highlight the area of effect using the **largest `radius`** among all effects in the spell. This gives the player a conservative view of the full potential area without displaying per-effect overlaps.

---

## 6. Spawner & Items (`src/spawner.rs`)

### 6.1 Tomes
- Remove `spawn_scroll` and update all item tables and UI references to use Tomes instead.
- Add `spawn_tome` (Solari/Nihil variant). Reading an unidentified Tome triggers the standard Magic Item Identification flow (DC = `10 + Spell Level`). On identification success, prompt to Study the Tome.
- Studying a Tome: roll a CHA check against `5 + 2 * Spell Level`. On success, add the spell to the player's `Spellbook` and display `"You learned [spell name]."`. On failure, destroy the Tome and display `"You failed to understand the Tome, and it crumbles."`

### 6.2 Shrines
- Add `spawn_shrine` (Solari/Nihil variant). Shrines render as `&` in their order's color.
- Add a `ShrineTried` component (or boolean field) to record that this shrine instance has already been attempted; attempting again immediately fails.
- Interaction flow:
    1. If `ShrineTried` is set, fail silently (or with a brief message).
    2. If the player's total max mana is already 5, fail with a message.
    3. Otherwise, roll CHA check against DC = `10 + total maximum mana (all colors)`.
    4. On failure: display `"The shrine is silent. Peace be with you."` and set `ShrineTried`.
    5. On success: display `"The shrine resonates with mystic energy. Raise your [Color] mana by one (1) point?"` and await confirmation. On confirm, increment `max_orange` or `max_purple` (matching shrine color) by 1 and set `ShrineTried`. On decline, do not set `ShrineTried` — the shrine remains available.

---

## 7. Target Resolution Logic

### 7.1 TargetSpec Types
- **`self`**: Auto-select caster.
- **`entity`**: Cycle between visible entities; returns the selected entity as the origin.
- **`location`**: Free cursor selection within `range`; returns the chosen tile as the origin.

In all cases, the area of effect is determined per-effect by `shape` and `radius` on each `EffectInstruction`.

---

## 8. Validation Plan

### 8.1 Unit Tests
- Test `ManaPool` capping and regeneration logic.
- Test `RawSpell` to `Spell` baking.
- Test that a spell with both non-zero orange and purple mana costs is rejected at bake time.
- Test that a level-0 spell (both costs zero) is rejected at bake time.
- Test `EffectInstruction` execution (damage, status application).

### 8.2 Integration Tests
- Verify full casting cycle: select spell -> target -> mana deducted -> effect applied -> turn ends.
- Verify Mana Drought application and recovery.
