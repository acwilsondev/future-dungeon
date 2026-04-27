# Implementation Plan: v0.10-content-pass

**Status:** APPROVED
**Epic:** v0.10-content-pass

This document outlines the technical changes required to transition the content
pipeline to a multi-file YAML-based system and expand the game's content with
lore-driven entries.

## 1. Data Pipeline Overhaul

### 1.1 Content Directory (`content/`)

- Create a `content/` directory at the project root.
- All YAML files in this directory (and subdirectories) will be loaded as part
  of the game's content.
- `content.json` will be deprecated and migrated to `content/base.yaml`.

### 1.2 Multi-file Loading Logic (`src/content.rs`)

- **Crate Swap:** Use `serde_yml` as a drop-in replacement for the unmaintained `serde_yaml`.
- **Merge Strategy:** Implement a recursive directory scanner to find all `*.yaml` files.
- **Collision Policy:** Treat duplicate entity names across different files as a **hard error** during loading to prevent silent overrides.
- Merge lists (`monsters`, `items`, `spells`) from all files into a single global `Content` struct.
- **Save Compatibility:** Save files can remain in JSON for now. We assume all existing saves are invalidated/deleted when upgrading to v0.10.

### 1.3 Feature Component Mapping (`src/spawner.rs`)

- Add `RawFeature` to support data-driven environmental features (stairs, doors, traps, shrines).
- **Component Composition:** Instead of hardcoded triggers, `RawFeature` will define a list of components to be composed onto the entity (e.g., `Door`, `Trap`, `Shrine`).
- Refactor `spawner.rs` to allow spawning any feature by its data-driven key or role.

### 1.4 Tagged Role System

- Add a `tags: Vec<String>` field to all `Raw` entity types.
- Standardize core tags: `door`, `trap`, `treasure`, `monster`, `hazard`, `shrine`.
- Implement a lookup method: `Content::query_by_tag(tag: &str, level: u16, branch: &Branch)`.
  - This returns a subset of entities that match the tag and satisfy floor/branch constraints.
  - Level generation will use the existing `spawn_chance` field as the weight for `select_weighted` on these subsets.

## 2. Hardcoding Audit & Externalization

### 2.1 Player Stats

- Move starting player stats (HP, attributes, starting gear) to a `player_defaults.yaml`.

### 2.2 Role-Based Spawning

- Refactor `spawn_environmental_features` and `spawn_room_features` to request objects by role rather than calling specific functions.
- Example: Instead of `spawn_door()`, the generator calls `spawn_by_tag("door")`.
- This allows branch-specific YAML files to define themed versions of core roles (e.g., "Vine Door" in Gardens).

### 2.3 Dungeon Rhythm & Floor Types

- Define floor archetypes in YAML (e.g., "Haven", "Boss Arena", "Standard").
- Each floor type should specify:
  - Spawn probabilities for monsters/items.
  - Mandatory features (e.g., Merchant on Floor 5).
  - Background color or special generation tweaks.

## 3. Lore-Driven Content Pass

### 3.1 Lore Snippets

- Define a `LoreSnippet` struct in `content.rs`:
  - `id`: Unique identifier.
  - `text`: The narrative content.
  - `faction`: Solari, Nihil, Aetheric, Biomass, or Iron.
- **Unlock System:** Implement a simple key-based unlock trigger (queryable interface) to track lore progress (e.g., `has_unlocked("first_nihil_pip")`).

### 3.2 New Entities

- Add 5-10 new monsters per major faction mentioned in the roadmap.
- Add 5-10 new items with lore-rich descriptions and unique component combinations.
- **Data Requirement:** Add a mandatory `description: String` to all `Raw` entity types.

## 4. Interaction: The "Look" Action

### 4.1 Keybinding Changes (`src/input.rs`)

- Remove support for `h`, `j`, `k`, `l` as movement keys (favoring arrow keys or numpad).
- Rebind `l` to trigger the `Look` state.

### 4.2 Look State Logic

- Implement a new `RunState::Look`.
- **Reuse Logic:** Use the existing targeting system (`RunState::ShowTargeting`) logic for cursor movement and boundary handling.
- When the cursor is over an entity:
  - Display its `Name` and `Description`.
  - Show relevant stats (HP, Power, Defense) if the entity is a monster.
  - Show the source YAML file if in debug mode.

## 5. Debug & Iteration Tools

### 5.1 Content Hot-Reloading

- Implement a `reload_content` command in the existing debug console (mapped to backtick `` ` ``).
- This clears existing lists and re-scans the `content/` directory.

### 5.2 Content Validation CLI

- Add a `--check-content` flag to the binary.
- Performs a **crate-free structural validation** by attempting to deserialze into `Raw` structs and calling their `validate()` methods.
- Checks for missing required items (e.g., Amulet) and broken references.

### 5.3 Floor Archetype Override

- Add `force_floor_type [ArchetypeID]` debug command to the console.
- Overrides the `level_gen.rs` logic for the next level transition.

## 6. Logging & Monitoring

### 6.1 Content Origin Tracing

- Add a `source_file: String` field to internal storage for diagnostic tracking.

### 6.2 Spawn Statistics Logger

- Output a debug summary after level generation: "Level 7 generated. Spawns: 14 Monsters, 6 Items, 1 Altar."

### 6.3 Performance Telemetry

- Log total time for `Content::load_from_dir`.
- Warn if total load time exceeds **200ms** to identify I/O bottlenecks or excessive content volume.

## 7. Tasks

- [x] Add `serde_yml` to `Cargo.toml`.
- [x] Implement `Content::load_from_dir(path: &Path)`.
- [x] Migrate `content.json` to `content/base.yaml`.
- [ ] Implement `RawFeature` with component composition and update `spawner.rs`.
- [x] Update `src/content.rs` tests to use directory-based loading.
- [ ] Externalize `spawn_player` stats.
- [ ] Refactor `level_gen.rs` to use role-based `spawn_by_tag` and floor archetypes.
- [x] Implement `RunState::Look` (leveraging targeting logic) and rebind `l`.
- [ ] Add lore snippet support with key-based unlock triggers.
- [x] Add initial content batch with mandatory descriptions and lore tags.
- [x] Implement debug console commands (`reload_content`) and `--check-content` CLI flag.
