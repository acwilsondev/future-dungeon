# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
make build      # cargo build
make run        # cargo run
make test       # cargo test
make lint       # cargo fmt --check + cargo clippy -D warnings
make harden     # complexity, test, unsafe/unwrap audit, coverage (tarpaulin)
make all        # build + test + lint
```

Run a single test: `cargo test test_name` (e.g. `cargo test test_save_load_game`)

The project uses `#![deny(clippy::all)]`, so all clippy warnings are compile errors. Always run `make lint` before committing.

## Workflow

Use Red-Green-Refactor for all fixes and features:

1. **Red** — write a failing test that demonstrates the bug or missing behaviour. Confirm it fails before writing any fix.
2. **Green** — make the minimal change to pass the test.
3. **Refactor** — clean up without breaking the test.

Never fix a bug without a test that would have caught it.

## Architecture

RustLike is a terminal roguelike using `ratatui`/`crossterm` for rendering and `hecs` for the ECS.

### Core loop (`src/main.rs`)
Runs at ~60fps. Each tick: render → poll input → map key to `Action` via `input::map_key_to_action` (keyed by current `RunState`) → `app.process_action()`. After player acts, `RunState` transitions to `MonsterTurn` and `app.monster_turn()` runs.

### App (`src/app/`)
`App` is the entire game state. It's split into domain-specific impl files:

| File | Responsibility |
|---|---|
| `mod.rs` | `App` struct definition, `new()` / `new_test(seed)` |
| `state.rs` | `RunState`, `MonsterAction`, `VisualEffect` enums |
| `actions.rs` | `process_action()` — dispatches `Action` enum to handlers |
| `actions_item.rs`, `actions_shop.rs`, `actions_alchemy.rs`, etc. | Domain-specific action handlers |
| `combat.rs` | Attack resolution |
| `level_gen.rs` / `level_gen_helpers.rs` | Procedural level generation, room feature placement |
| `level_transition.rs` | Stair traversal, level caching in `app.levels` |
| `monster_ai_calc.rs` / `monster_ai_execute.rs` | Monster decision-making and movement |
| `monster_perception.rs` | FOV/hearing/alert state |
| `serialization.rs` / `snapshot.rs` | Pack/unpack `hecs::World` to `Vec<EntitySnapshot>` for save/load |
| `player_move.rs` | Player movement, bump attacks, trap triggering |
| `world_update.rs` | End-of-turn state updates (poison, confusion, light decay) |
| `visual_effects.rs` | Particle/projectile effects |

### ECS pattern
All game objects are `hecs` entities with component bundles. Because `hecs::World` is not serializable, `pack_entities()` drains it into `Vec<EntitySnapshot>` (a tagged-component list) and `unpack_entities()` rebuilds it. This happens on save/load and level transitions.

### Content system (`src/content.rs` + `content.json`)
Monsters, items, and their stats are defined in `content.json` as `RawMonster`/`RawItem` structs. `Content::load()` deserializes this at startup. `spawner.rs` reads `Content` to instantiate entities. To add a new monster or item, edit `content.json` — no code change needed unless adding new fields.

### RunState machine
`RunState` in `src/app/state.rs` controls which UI panel renders and which keys are active. `input.rs` matches `(RunState, KeyCode)` to `Action`. Most UI state (cursor positions, active menus) is held as skipped-during-serde fields on `App`.

### Persistence (`src/persistence.rs`)
Saves to `savegame.json` in the working directory. `load_game()` deletes the file immediately after reading (roguelike iron-man style). Death also deletes the save via `delete_save()`.

### Branching dungeon
`Branch` enum (`Main`, `Gardens`, `Vaults`) and `dungeon_level` together key the `app.levels` cache. Stairs encode their `destination: (u16, Branch)` directly as a component.
