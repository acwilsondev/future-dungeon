# Changelog

All notable changes to this project will be documented here.

## [Unreleased]

### Fixed

- Replaced panic-on-failure `.unwrap()` calls with graceful `let...else` early returns in `apply_class_selection`, and the `spawn`, `heal`, and `levelup` debug console commands (`src/app/actions.rs`). These would have crashed the game if the player entity was unexpectedly absent.
- Changed `world.insert_one(...).unwrap()` calls in `apply_class_selection` to `.ok()` — insertion can only fail if the entity no longer exists, which is not recoverable mid-selection anyway.
- Replaced `rolls.iter().min().unwrap()` in `resolve_attack` with `.expect("rolls is never empty")` to document the invariant rather than leaving an unexplained unwrap (`src/app/combat.rs`).
- Added `Map::idx(x, y) -> Option<usize>` bounds-checking helper (`src/map.rs`) and replaced two direct unchecked index calculations in `monster_ai_execute.rs` and `monster_ai_calc.rs` that would panic when a monster stood on the map's bottom or right edge.
- `Content::load()` now returns `anyhow::Result<Self>` instead of panicking on a missing or malformed `content.json`. Added `load_from_path` and `load_from_str` helpers for testability. Error propagates through `App::new()` to `main`.
