# SOLID Refactor Plan

## Root Cause

Most violations share one pattern: code dispatches to behavior by **checking what a thing is** rather than **calling what a thing does**. Every new type requires editing existing dispatch sites instead of adding new code alongside existing code.

The ECS architecture is sound and stays. The issue is in the logic layered on top of it.

---

## Priority 1 — Item Effect Dispatch (High OCP gain, Medium risk)

**File:** `src/app/items_use_basic.rs`

**Problem:** `use_item()` is a sequential chain of `if let Some(component) = world.get::<ComponentType>(item_id)` checks. Adding a new item effect means adding another branch to an existing function.

```rust
// current: add new effects by editing this chain
if let Some(heal) = self.world.get::<&Potion>(item_id) { ... }
if let Some(poison) = self.world.get::<&Poison>(item_id) { ... }
if let Some(strength) = self.world.get::<&Strength>(item_id) { ... }
// ...
```

**Fix:** Lean into ECS. Each effect type is handled by a focused function triggered by its component's presence. Adding a new effect means adding a new query + handler function — not touching the dispatch.

```rust
// target: each effect is self-contained, dispatch is data-driven
fn apply_item_effects(world, item_id, target_id) {
    apply_heal_effect(world, item_id, target_id);
    apply_poison_effect(world, item_id, target_id);
    apply_strength_effect(world, item_id, target_id);
    // adding new: add new function + call here, no branching
}
```

**Steps:**
- [x] Extract each if-let block into a standalone `apply_xxx_effect(world, item_id, target_id)` function
- [x] Replace the chain in `use_item()` with sequential calls to each handler
- [x] Verify each handler early-returns cleanly if the component is absent
- [x] Run `make test` + `make lint`

---

## Priority 2 — Feature Spawning (High OCP gain, Low risk)

**File:** `src/spawner.rs` `spawn_feature()`, `src/content.rs` `RawFeatureKind`

**Problem:** Every new dungeon feature requires a new `RawFeatureKind` variant, a new match arm, and potentially new component + interaction logic.

```rust
// current: adding a new feature type edits this match
match &raw.kind {
    RawFeatureKind::Door => ...,
    RawFeatureKind::Trap { damage } => ...,
    RawFeatureKind::PoisonTrap { damage, turns } => ...,
    RawFeatureKind::Cover => ...,
}
```

**Fix:** Flatten `RawFeatureKind` into a struct with optional fields. The spawner reads fields generically rather than dispatching on variant.

```rust
// target: data-driven, no match arm needed for new feature types
pub struct RawFeature {
    pub blocks: bool,
    pub damage: Option<i32>,
    pub poison_turns: Option<u32>,
    pub cover: bool,
    // new fields added here with Option<T> — no match arm needed
}
```

**Steps:**
- [x] Replace `RawFeatureKind` enum variants with a flat `RawFeature` struct
- [x] Update YAML content files to use flat fields
- [x] Rewrite `spawn_feature()` to read struct fields instead of matching on variant
- [x] Run `make test` + `make lint`

---

## Priority 3 — Floor Ritual Table (Medium OCP gain, Low risk)

**File:** `src/app/level_gen.rs`

**Problem:** New dungeon events require editing `generate_level()` directly. Magic numbers live in code.

```rust
// current: every new floor event edits this function
if self.dungeon_level % 10 == 5 { /* merchant floor */ }
if self.dungeon_level.is_multiple_of(20) { /* reset shrine */ }
if self.dungeon_level == 10 { /* amulet */ }
```

**Fix:** Define floor events in content YAML. `generate_level()` reads the table and applies matching events — never needs editing for new floor types.

```yaml
# content/floors.yaml
floor_events:
  - trigger: { every: 10, offset: 5 }
    kind: MerchantHaven
  - trigger: { every: 20 }
    kind: ResetShrine
  - trigger: { at: 10 }
    kind: AmuletSpawn
```

**Steps:**
- [x] Define `RawFloorEvent` and `FloorTrigger` types in `content.rs`
- [x] Add `floor_events` field to `Content`, loaded from `floor_events.yaml`
- [x] Replace the if-chain in `spawn_room_features()` with a loop over matching events
- [x] Run `make test` + `make lint`

---

## Priority 4 — Personality Thresholds to Data (Medium OCP gain, Low risk)

**File:** `src/app/monster_ai_calc.rs`, `src/content.rs` `RawMonster`

**Problem:** Adding a new personality requires adding branches in `decide_ai_action()`. Magic numbers (`4.0`, `0.5`) are hardcoded.

```rust
// current: magic numbers and personality branches in logic
if ctx.personality == Personality::Cowardly && hp < max_hp / 2 { ... }
if ctx.personality == Personality::Tactical && dist < 4.0 { ... }
```

**Fix:** Move thresholds to `RawMonster` fields. The calc function reads monster-specific values rather than branching on personality name. Personality enum can remain for display/lore but stops driving behavior logic.

```yaml
# content/monsters.yaml — each monster carries its own thresholds
- name: Goblin
  flee_below_hp_pct: 0.5
  preferred_range: null
- name: Archer
  flee_below_hp_pct: null
  preferred_range: 4.0
```

**Steps:**
- [x] Add `AIThresholds` component to `components.rs` with `from_personality()` constructor
- [x] Add `AIThresholds` to `EntitySnapshot` + pack/unpack in serialization (backwards-compat fallback included)
- [x] Spawn `AIThresholds` alongside `AIPersonality` in `spawn_monster()`
- [x] Replace personality branches in `decide_ai_action()` with threshold field reads
- [x] Run `make test` + `make lint`

---

## Priority 5 — Split `actions.rs` (SRP, Low risk)

**File:** `src/app/actions.rs` (612 lines)

**Problem:** Class selection logic (123 lines), main menu logic, and debug console live inside `actions.rs`. The file pattern for splitting already exists (`actions_item.rs`, `actions_shop.rs`).

**Steps:**
- [ ] Extract main menu handler into `actions_menu.rs`
- [ ] Extract class selection handler into `actions_class_select.rs`
- [ ] Extract debug console handler into `actions_debug.rs`
- [ ] Leave only the top-level `process_action()` dispatch in `actions.rs`
- [ ] Run `make test` + `make lint`

---

## Priority 6 — Split `content.rs` (SRP, Low risk)

**File:** `src/content.rs` (1211 lines)

**Problem:** Raw type definitions, disk I/O, and validation are mixed. Testing validation requires disk I/O.

**Steps:**
- [ ] Move `RawMonster`, `RawItem`, `RawSpell`, etc. into `src/content/types.rs`
- [ ] Move `load_from_dir()` and file I/O into `src/content/loader.rs`
- [ ] Move semantic validation into `src/content/validate.rs`
- [ ] Re-export public API from `src/content.rs` (or `src/content/mod.rs`)
- [ ] Run `make test` + `make lint`

---

## Priority 7 — Decompose `combat.rs` (SRP, Medium risk)

**File:** `src/app/combat.rs` (925 lines)

**Problem:** Hit/miss calculation, damage rolls, crits, status-on-hit, and stat mutations are all in `resolve_attack()`. Hard to test each concern independently.

**Steps:**
- [ ] Extract hit resolution into `fn roll_hit(attacker, defender) -> HitResult`
- [ ] Extract damage calculation into `fn roll_damage(attacker, hit) -> i32`
- [ ] Extract on-hit effect application into `fn apply_hit_effects(world, attacker, target, hit)`
- [ ] Have `resolve_attack()` compose these three functions
- [ ] Run `make test` + `make lint`

---

## Summary

| Priority | Change | OCP gain | SRP gain | Risk |
|---|---|---|---|---|
| 1 | Item effect dispatch | High | Medium | Medium |
| 2 | Feature spawning | High | Low | Low |
| 3 | Floor ritual table | Medium | Low | Low |
| 4 | Personality thresholds | Medium | Low | Low |
| 5 | Split `actions.rs` | Low | High | Low |
| 6 | Split `content.rs` | Low | High | Low |
| 7 | Decompose `combat.rs` | Low | High | Medium |

Priorities 2–4 are pure data-driven changes with no architectural risk — good candidates to batch in one branch. Priorities 5–6 are mechanical file splits. Priority 1 and 7 touch live gameplay logic and should each get their own branch with focused testing.
