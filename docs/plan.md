# Refactor Plan

Fixes identified in code review, ordered by priority.

---

## Critical — Panic Risks

- [ ] **Unwrap in hot paths** — Replace `.unwrap()` with `let...else` returns in `src/app/actions.rs:124`, `src/app/turn_tick.rs:260+`, `src/app/actions_shop.rs:141-143`, and other ECS query sites in `combat.rs` and `monster.rs`.
- [ ] **Map index bounds check** — Add a helper `fn map_idx(&self, x: u16, y: u16) -> Option<usize>` and use it in `src/app/monster_ai_execute.rs:19-20` and `monster_ai_calc.rs:110` instead of direct indexing.
- [ ] **Content loading panic** — Change `Content::load()` in `src/content.rs:92-97` to return `Result` and propagate to `main` instead of using `.expect()`.

---

## Architecture

- [ ] **Hardcoded item name** — Add a `Levitation` component to `src/components.rs` and replace the `if name == "Boots of Levitation"` string check in `src/app/player_move.rs:135-143`. Audit for other hardcoded item name checks.
- [ ] **God object `App`** — Split `src/app/mod.rs` into `GameState` (world, map, levels), `UIState` (cursors, active menus), and `PlayerStats` (kills, turn count, flags). Long-term; coordinate with serialization changes.

---

## Performance

- [ ] **Unused Dijkstra allocation** — Remove the `DijkstraMap` construction in `src/app/world_update.rs:77` and keep only the BFS that follows it.
- [ ] **`to_string()` inside filter** — Hoist `branch_str.to_string()` out of the closure in `src/app/level_gen.rs:228`.
- [ ] **Weighted selection duplication** — Extract the repeated weighted-random loop in `src/app/level_gen.rs` into a shared helper `fn select_weighted(...)`.

---

## Correctness

- [ ] **Light fade magic number** — Define `const TORCH_FADE_TURNS: i32 = 1000;` and replace the `== 1001` check in `src/app/turn_tick.rs:28`. Verify intended value.
- [ ] **Targeting cursor bounds** — Clamp `targeting_cursor` to `map.width`/`map.height` when updating it in `src/app/actions.rs:75-120`.

---

## Test Coverage

- [ ] **Escape spawning** — Add a test for the `if self.escaping` branch in `src/app/level_gen.rs:108-161` that verifies monster count increases.
- [ ] **Combat edge cases** — Add tests to `src/app/combat.rs` for 0-defense targets and overkill damage.
- [ ] **Content loading failure** — Add a test for malformed/missing `content.json` once `Content::load()` returns `Result`.
