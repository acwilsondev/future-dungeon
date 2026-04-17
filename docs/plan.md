# Plan

Improvements from the April 2026 code audit, ordered by priority.

---

## Panic Risks

- [x] **Unwrap on `get_player_id()` in hot paths** — Replace `.unwrap()` with `let...else` returns in `src/app/player_move.rs:36`, `src/app/actions_respec.rs:7`, and `src/app/actions_respec.rs:56`. All other call sites already use the safe pattern.
- [x] **Unchecked `map.blocked` index** — `src/app/player_move.rs:296` indexes directly without an upper-bound guard. Use `self.map.idx(new_x, new_y).map(|i| !self.map.blocked[i]).unwrap_or(false)` to match the safe pattern used everywhere else.

---

## Duplication

- [x] **Duplicate `App` constructors** — `App::new_random()` and `App::new_test()` list every field twice with two lines of difference. Extract a private `App::build(seed: u64, content: Content) -> Self` that both call (`src/app/mod.rs:123–217`).
- [x] **Duplicate spawner functions** — `spawn_item` and `spawn_item_in_backpack` share a 60+ line body differing only in one component. Extract `fn add_item_components(cb: &mut EntityBuilder, raw: &RawItem)` (`src/spawner.rs:133–214, 416–501`).
- [x] **Duplicate attribute match blocks** — `handle_level_up_input` and `handle_respec_input` contain identical 6-arm attribute increment blocks. Extract `fn increment_attribute(&mut self, player_id: Entity, cursor: usize)` (`actions_levelup.rs:21–46`, `actions_respec.rs:58–87`).
- [x] **Duplicate monster attack outcome handling** — Melee and ranged attack arms in `execute_monster_action` share identical ~25-line post-attack blocks. Extract `fn handle_attack_outcome(&mut self, ...)` (`src/app/monster_ai_execute.rs:40–148`).

---

## Architecture

- [x] **Magic string item effects** — `"Ring of Regeneration"` is matched by name in `src/app/turn_tick.rs:87`. Add a `Regeneration` component applied by the spawner so effects are data-driven and rename-safe. Audit for other magic-string item checks.
- [x] **`shop_mode: usize`** — Replace the `// 0 = Buy, 1 = Sell` magic number with `enum ShopMode { Buy, Sell }` (`src/app/mod.rs:89`).
- [ ] **God object `App`** — Long-term: split `src/app/mod.rs` into `GameState` (world, map, levels), `UIState` (cursors, active menus), and `PlayerStats` (kills, turns, flags). Coordinate with serialization.

---

## Correctness

- [x] **`get_modifier` floor division** — `(score - 10) / 2` truncates toward zero in Rust; a score of 9 returns 0 instead of -1. Use `.div_euclid(2)` for D&D-correct floor behavior (`src/components.rs`).
- [x] **`select_weighted` float rounding** — Fallback to index 0 when the loop exhausts biases the first item. Always iterate all-but-last items and return `items.last()` as the guaranteed fallback (`src/app/level_gen.rs:8–21`).

---

## Performance

- [ ] **5 world scans per player step** — `get_interactable_at` runs 5 separate O(n) queries per movement. Consolidate into one query (`src/app/player_move.rs:6–33`).
- [ ] **Redundant world queries in combat** — Several equipment helpers run separate full-world iterations; one nested query fetches `InBackpack` twice. Consolidate into joined queries (`src/app/combat.rs:23–275`).
- [x] **`extend` with clone** — `monster_spawns.extend(mb.monster_spawns.clone())` should be `extend_from_slice(&mb.monster_spawns)` (`src/app/level_gen.rs:111`).

---

## Robustness

- [x] **`content.json` hardcoded relative path** — `Content::load()` silently fails if not run from the project root. Existing `load_from_path`/`load_from_str` API already supports this; noted for future embed-via-`include_str!` improvement.
- [x] **No content validation** — `"Amulet of the Ancients"`, `"Identification Scroll"` are mandatory named items with no existence check at load time. Added `Content::validate()` called from `load_from_str`.
- [x] **Unbounded log `Vec`** — `self.log` grows forever. Trimmed to 500 entries each turn tick (`src/app/turn_tick.rs`).

---

## Test Coverage

- [x] **`select_weighted` distribution** — Added `test_select_weighted_last_item_selectable` and `test_select_weighted_first_item_selectable`.
- [ ] **`level_transition.rs`** — No tests for level transition logic at all.
- [x] **`get_modifier` floor behavior** — Added `test_get_modifier_floor_division` verifying score 9 → -1.
- [x] **Content validation** — Added `test_missing_required_item_returns_err`.
