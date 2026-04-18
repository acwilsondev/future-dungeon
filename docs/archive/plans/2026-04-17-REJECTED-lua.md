# Lua Scripting: Feasibility Report

## Summary

Extracting gameplay features to Lua is feasible but non-trivial. The codebase has clear
functional boundaries, all components are already serializable, and the two-phase monster
AI is an ideal scripting seam. The main obstacle is the ECS: `hecs` entities are opaque
Rust handles, so Lua can't touch them directly. A thin binding layer is needed everywhere.

**Rough complexity: medium.** A focused effort targeting combat + AI + item effects could
be done in a few weeks. Exposing level generation and the full ECS would be a longer
project.

---

## What's Already Data-Driven

`content.json` defines monsters and items, including spawn weights, level ranges,
biome/branch filters, boss phases, and equipment stats. This is the existing
"scripting" layer. It covers _what_ exists in the world but not _how_ anything behaves.

Combat formulas, AI decision trees, status effect tick logic, and level generation
parameters are all hardcoded in Rust.

---

## Architecture Snapshot

```sh
src/actions.rs           27 lines   Action enum (player intent)
src/components.rs       357 lines   ~60 ECS component types
src/app/
  actions.rs            406 lines   Action dispatcher / RunState machine
  combat.rs             521 lines   Attack resolution
  monster_ai_calc.rs    431 lines   AI decision logic
  monster_ai_execute.rs 517 lines   AI action execution
  items_use_basic.rs    308 lines   Consumables, equipment
  items_use_ranged.rs   611 lines   Ranged/AoE targeting
  turn_tick.rs          459 lines   Status effects, torch decay
  level_gen.rs          359 lines   Spawn logic, level rhythm
  player_move.rs        688 lines   Movement, traps, interactions
```

The game state lives in `App`, which holds a `hecs::World`, the current `Map`, an RNG,
and transient UI cursors. There's no event bus or callback system today — input flows
through `process_action()` synchronously.

---

## The Binding Problem: hecs

`hecs` entities are integer handles with no runtime type information. Lua would see them
as opaque numbers and would need a helper API to read and write components:

```lua
local hp = game.get_component(entity_id, "CombatStats").hp
game.set_component(entity_id, "CombatStats", { hp = hp - 5 })
```

Every component read/write crosses the FFI boundary, so the binding layer needs to map
component names to Rust types and perform (de)serialization. Because all components
already `#[derive(Serialize, Deserialize)]`, `serde_json` can bridge the gap without
hand-writing every accessor — but this adds a JSON round-trip per access in hot paths.

---

## Systems by Complexity

### Tier 1 — Straightforward (1–2 days each)

**Status effects** (`turn_tick.rs`)
Poison, Confusion, Strength, and Speed are plain data components with a countdown.
Exposing `apply_status(entity, kind, params)` and `tick_status(entity)` as Lua hooks is
low risk. The existing component structs map cleanly to Lua tables.

**Action dispatch** (`src/actions.rs`, `src/app/actions.rs`)
`enum Action` has ~25 variants. Lua could construct actions as tagged tables and hand
them to `process_action()`. The RunState machine is already the bottleneck — adding a
Lua call before the match arm is minimal.

**Data queries**
`content.json` is already deserialized into `Vec<RawMonster>` / `Vec<RawItem>`.
Read-only access from Lua (e.g. to look up a monster's stats before spawning) is free.

---

### Tier 2 — Moderate (3–5 days each)

**Combat** (`combat.rs`)
`resolve_attack()` is a self-contained function with clear inputs (attacker, target,
weapon, flags) and a structured `AttackResult` output. The attack roll, damage formula,
and critical logic could each be replaced with a Lua callback:

```lua
game.on_resolve_attack(function(attacker, target, ctx)
  ctx.roll = game.roll_d20() + ctx.attr_mod
  ctx.damage = game.roll_dice(ctx.n_dice, ctx.die_type) - ctx.target_av
  return ctx
end)
```

The main friction is that `resolve_attack` currently borrows `&mut self` throughout, so
splitting it into hookable steps requires restructuring the function signature.

**Item effects** (`items_use_basic.rs`, `items_use_ranged.rs`)
Item use is dispatched by matching component presence (`Potion`, `Ranged`, `Confusion`,
etc.). A Lua hook per item type is natural:

```lua
game.on_use_item("Potion", function(user, item, ctx)
  game.heal(user, ctx.heal_amount)
end)
```

The complication is that some item effects have multiple steps (targeting cursor, fire
projectile, apply AoE, apply status) spread across two files and 900+ lines. Mapping
this to clean hooks requires refactoring the effect chain first.

**Monster AI** (`monster_ai_calc.rs`, `monster_ai_execute.rs`)
The two-phase split (calculate decision → execute action) is already the right shape.
`MonsterAction` is a small enum (Move, AttackMelee, AttackRanged, Wait, Flee). Replacing
the calculate phase with a Lua callback is realistic:

```lua
game.on_monster_ai(function(monster_id, ctx)
  if ctx.hp_pct < 0.3 then
    return { action = "Flee" }
  end
  return { action = "AttackMelee", target = ctx.nearest_enemy }
end)
```

The blocker is that `monster_ai_calc.rs` does its own FOV/pathfinding queries inline.
Those helpers (`is_visible`, `distance_to`, `line_clear`) would need to be exposed to
Lua before the AI callback is useful.

---

### Tier 3 — Significant Work (1–2 weeks)

**Level generation hooks** (`level_gen.rs`, `map_builder.rs`)
`generate_level()` is a single long function; `MapBuilder` builds room geometry in
`map_builder.rs` (621 lines). Adding spawn hooks (e.g. "after rooms are placed, before
monsters spawn") is plausible. Giving Lua control over room layout or tile placement is
a deep refactor of `MapBuilder`.

**Full ECS access**
Exposing entity creation, component insertion, and arbitrary queries from Lua requires
a registry that maps string names to Rust types. This is the heaviest part of any Lua
integration. The `snapshot.rs` serialization format (which already lists every component)
is a natural starting point for the registry, but the impedance mismatch between Lua
tables and Rust structs with optional fields adds friction.

---

## Crate Options

The two main Lua crates for Rust are:

| Crate | Pros | Cons |
|-------|------|------|
| **mlua** | Actively maintained, async support, good error handling | Slightly more API surface |
| **rlua** | Older, battle-tested | Less maintained, no async |

`mlua` is the better choice today. It supports Lua 5.4, LuaJIT, and Luau, and integrates
well with `serde` for automatic table ↔ struct conversion (via `mlua::serde`), which is
the critical feature for mapping components without hand-writing every accessor.

---

## What a Minimal Integration Would Look Like

A low-risk first step: Lua-scriptable item effects only.

1. Add `mlua` to `Cargo.toml`.
2. Create a `Lua` instance in `App` (stored alongside `content`).
3. Load a `scripts/items.lua` at startup.
4. In `items_use_basic.rs`, before the component-match dispatch, check whether the item
   has a `script` field in `RawItem` (add `script: Option<String>` to the content
   schema). If present, call the named Lua function with the entity IDs.
5. Expose `game.heal(entity, amount)`, `game.apply_status(entity, kind, params)`, and
   `game.log(msg)` as the initial Lua API.

This touches ~5 files, adds no new abstraction over the ECS, and is fully opt-in — items
without a `script` field behave identically to today.

---

## Risks

**Determinism.** The game uses a seeded ChaCha8 RNG. Lua scripts that call `game.roll_*`
helpers need to go through the same RNG instance, or replays and seeded runs break.

**Borrow checker friction.** Most `App` methods take `&mut self`. Passing a mutable
reference into a Lua callback while also letting the callback call other `App` methods
is a borrow conflict. The standard fix is to extract the Lua state into a separate
`LuaState` struct that holds the `mlua::Lua` instance and is called through an
`Arc<Mutex<...>>` — but this adds overhead and complexity.

**Save compatibility.** If Lua scripts add new components, the snapshot format needs to
accommodate arbitrary component data. The current `EntitySnapshot` struct is a fixed
list. A `HashMap<String, serde_json::Value>` extension field would handle this.

**Performance.** FFI crossings are cheap, but a Lua callback on every monster's AI tick
(potentially 20–30 monsters per turn) adds up. Profile before committing to per-entity
Lua AI.

---

## Revised Plan (Post-Review)

The senior review identified 7 concrete problems with the original minimal-integration
proposal. This section revises the plan to address each one.

### Core Architecture Change: Command Buffer

The original plan let Lua call `game.heal()`, `game.set_component()`, etc. directly.
This causes borrow-checker panics the moment any callback re-enters the ECS while it is
already mutably borrowed by the calling system. The entire mutation model needs to change.

**Principle: Lua observes, queues, never writes.**

Lua functions receive read-only snapshots and return a list of commands. Rust applies
the commands after the Lua call returns and all borrows are released.

```lua
-- Lua: returns a command, does not mutate
game.on_resolve_attack(function(ctx)
  if ctx.attacker.hp_pct > 0.8 then
    return { type = "Damage", target = ctx.target_id, amount = ctx.damage * 1.5 }
  end
  return { type = "Damage", target = ctx.target_id, amount = ctx.damage }
end)
```

```rust
// Rust side: call Lua, collect commands, then apply
let cmds: Vec<GameCommand> = lua_ctx.call_hook("on_resolve_attack", &snapshot)?;
for cmd in cmds {
    cmd.apply(&mut world);  // safe: no active borrows
}
```

`GameCommand` is a Rust enum covering the legal mutation surface: `Damage`, `Heal`,
`ApplyStatus`, `SpawnEntity`, `Log`, `RemoveEntity`. Lua cannot issue arbitrary ECS
mutations — only named, validated commands.

---

### Addressing Each Critique

**1. JSON Bridge Performance**

The `serde_json` round-trip was a mistake. The correct approach is `mlua::UserData`
with manually implemented `get`/`set` methods for components that appear in hot paths
(primarily `CombatStats`, `Position`, `AlertState`). Cold-path components (equipment
details, boss phases) can still use `serde` conversion since they aren't queried per
tick.

Rule of thumb: if a component is read inside the monster turn loop, give it a
`UserData` impl. Everything else can serialize.

**2. Reentrancy / Borrow Checker**

Addressed entirely by the Command Buffer above. Lua can never hold a live reference to
the ECS. The Lua call receives a `LuaSnapshot` struct (plain data, no borrows), and
returns commands. This is safe to call even from inside `on_turn_tick`.

**3. Two-Phase AI: Spatial Query Cost**

Rather than exposing `is_visible()` / `distance_to()` as individual FFI calls, pass a
`MonsterContext` snapshot containing pre-computed values that the AI commonly needs:

```rust
struct MonsterContext {
    entity_id: u64,
    hp_pct: f32,
    position: (u16, u16),
    nearest_enemy_id: Option<u64>,
    nearest_enemy_dist: f32,
    nearest_enemy_visible: bool,
    alert_state: &'static str,  // "Sleeping" | "Curious" | "Aggressive"
}
```

Rust computes this once per monster before the Lua call. Lua can make its decision
from the context without any further FFI crossing. Complex cases (custom pathfinding)
that truly need spatial queries can call a `game.query_spatial()` function, but this
is opt-in and understood to be slower.

**4. Determinism: RNG**

Two changes:

- The game RNG (`ChaCha8Rng`) is never exposed to Lua.
- A separate `LuaRng` is derived from a sub-seed at world creation:
  `lua_seed = main_seed ^ 0xDEADBEEF_DEADBEEF`. Lua calls `game.roll_d20()` /
  `game.roll_dice(n, d)` which advance `LuaRng` independently.

This makes Lua rolls deterministic without coupling them to the main RNG sequence.
Lua's `pairs()` over the snapshot structs is deterministic because those structs are
arrays, not hash maps.

**5. Save Compatibility**

Rather than an untyped `HashMap<String, serde_json::Value>`, scripts register their
schema at load time:

```lua
game.register_component("Mana", { current = "int", max = "int" })
```

The Rust registry validates this at startup and adds a typed extension slot to
`EntitySnapshot`. On load, unknown fields are errors, not silent data corruption.
This is more work up front but prevents the versioning nightmare.

**6. Infinite Loops**

`mlua` exposes `set_hook` with an instruction counter. Set a per-call budget (e.g.
100,000 instructions) that's enough for any reasonable AI decision but would catch
an infinite loop within milliseconds. Script errors surface as logged warnings and
fall back to the default Rust behaviour — they never hang the game.

**7. Hot Reloading**

Transient state (running status effects, current HP, alert states) lives in the ECS,
not in Lua. Scripts only contain functions, not state. Reloading a script therefore
means:

1. Re-read the `.lua` file from disk.
2. Re-register all hooks in the fresh Lua environment.
3. The ECS is untouched.

This is safe because Lua is stateless between calls by design. The one exception is
Lua-registered component schemas (point 5) — reloading a script that changes a schema
requires a level reload, which is an acceptable constraint.

---

### Revised Minimal Integration

The original "item effects only" scope is still the right starting point, but the
implementation changes:

1. Add `mlua` to `Cargo.toml` with instruction limits enabled.
2. Add a `LuaState` struct to `App` holding the `Lua` instance and a `LuaRng`.
3. Define `GameCommand` enum covering the mutation surface Lua needs.
4. Add `script: Option<String>` to `RawItem` in the content schema.
5. In `items_use_basic.rs`, when an item has a script: build a read-only `ItemUseCtx`
   snapshot, call the Lua function, collect returned `GameCommand`s, apply them.
6. Expose `game.roll_dice`, `game.log`, and nothing else initially. Commands only.

No direct ECS writes from Lua. No `serde_json` in the item-use path. Sandboxed RNG.
Instruction-limited execution.

---

# Senior Engineer Review: Adversarial Critique

## 1. The Performance Tax (The "JSON Bridge" Fallacy)

The proposal to use `serde_json` for component access is a major red flag for performance.

- **The Math:** If 30 monsters each have 5 components and the Lua AI checks 3 of them per tick, that’s **450 JSON serialization/deserialization cycles per turn**. In a turn-based game, this creates noticeable "hitch" or latency during the enemy turn.
- **Adversarial Requirement:** We must avoid string-based serialization in hot paths. If we aren't using `mlua::UserData` or direct pointer offsets via a trait-based registry, we are building a performance bottleneck by design.

## 2. Reentrancy & The Borrow Checker Nightmare

This is the single biggest risk to project stability.

- **The Trap:** If `combat.rs` calls a Lua hook `on_damage`, and that Lua hook calls `game.heal()`, the `heal` function will likely try to borrow the `World` which is already mutably borrowed by the combat system.
- **The Consequence:** This leads to `RefCell` panics or necessitates a "Deferred Command Buffer" pattern.
- **Adversarial Requirement:** We cannot simply "extract the Lua state." We need a strict **Command Queue** where Lua _requests_ changes instead of _applying_ them, or we face a total rewrite of our core loop.

## 3. The "Two-Phase" AI Illusion

The proposal assumes the "Calculate" phase is a clean entry point for scripting.

- **Reality Check:** `monster_ai_calc.rs` currently does inline FOV and pathfinding. If we move the decision to Lua but leave the spatial queries in Rust, we are crossing the FFI boundary dozens of times per monster just to ask "Can I see the player?".
- **Adversarial Requirement:** We must profile the cost of 100+ FFI calls for pathfinding queries per turn. If we don't batch these queries or pass a "Spatial Snapshot" to Lua, the FFI overhead will dwarf the actual logic execution.

## 4. Determinism & RNG Leaks

- **The Risk:** If Lua can iterate over a `pairs()` table (which is non-deterministic in some Lua versions) and calls the RNG based on that order, the seed is blown and replays break.

- **Adversarial Requirement:** We must provide a **Hard-Sandboxed RNG API** to Lua that is decoupled from the core game RNG but initialized from a sub-seed.

## 5. Save Compatibility & Type Safety

The suggested `HashMap<String, serde_json::Value>` turns our strictly typed Rust game into a "stringly-typed" maintenance nightmare.

- **The Versioning Problem:** If a Lua script from v0.1 adds a `mana` field to a `CombatStats` table, and v0.2 changes that script, how do we handle the migration of that opaque JSON blob?
- **Adversarial Requirement:** We need a **Schema Registry** that Lua must register with at load time to prevent save corruption.

## 6. Stability: The "Infinity Loop" Problem

The report ignores execution safety.

- **Adversarial Requirement:** We _must_ use `mlua`'s instruction limit features. A game should never hang forever because of a logic error in a `.lua` data file.

## 7. Hot Reloading: The Missing MVP Feature

The primary justification for Lua is iteration speed. If we have to restart the game to test a damage formula change, Lua is just "Rust with worse types."

- **Adversarial Requirement:** The proposal must include a strategy for **State-Preserving Script Reloads**. How do we reload `items.lua` without losing the transient status effects currently running on 50 entities?

---

### Final Verdict

**Recommendation:** **REJECT** the "Minimal Integration" (Item Effects) as proposed. It encourages the use of `serde_json` as a crutch and ignores the reentrancy panics that will inevitably occur once scripts get complex.

**Counter-Proposal:** Start with a **read-only Data View** for Lua. Let Lua calculate a "Damage Multiplier" or "AI Target" based on a read-only snapshot, returning a primitive value. Do not allow Lua to call `game.heal()` or `game.set_component()` directly until a **Command Buffer** architecture is implemented.

---

# Senior Engineer Review: Round 2 (Mitigation Audit)

The architect's move to a **Command Buffer** and **Read-Only Snapshots** is a significant improvement that resolves the most dangerous reentrancy and borrow-checker issues. However, the proposed mitigations introduce new "architectural gravity" that we must account for.

## 1. The "Snapshot Bloat" Risk
>
> **Architect's Solution:** Pre-compute common values in a `MonsterContext` to avoid FFI calls.

- **Adversarial Critique:** This is a classic "eager vs. lazy" problem. If we pre-compute FOV, nearest enemy, and pathfinding for 30 monsters every turn, but only 2 monsters are actually scripted in Lua, we are wasting 90% of that computation.
- **Requirement:** We need a **Lazy Snapshot** or a **Query Cache**. Rust should only compute `nearest_enemy_visible` the first time Lua asks for it, and then cache it for the remainder of that monster's tick. A monolithic "everything Lua might need" struct will kill performance just as surely as FFI overhead.

## 2. Command Buffer Atomicity & Validation
>
> **Architect's Solution:** Lua returns a list of `GameCommand` enums.

- **Adversarial Critique:** What happens if Lua returns `[{type: "Damage", target: 5}, {type: "Heal", target: 5}]`? Or worse, a command that references an entity that was destroyed by a _previous_ command in the same list?
- **Requirement:** The `CommandProcessor` must handle **Command Validation** and **Entity Liveness**. We cannot blindly apply a list of commands. Each command application must re-verify that the target entity still exists and is in a valid state for that operation.

## 3. The "Schema Registry" maintenance burden
>
> **Architect's Solution:** Lua registers schemas like `game.register_component("Mana", ...)`.

- **Adversarial Critique:** This creates a dual-maintenance burden. If we decide to move a Lua-prototyped component into Rust for performance, we have to rewrite the registration logic, the snapshot logic, and the save-file migration logic.
- **Requirement:** We need a **Unified Data Definition (UDD)**. Whether a component is in Rust or Lua, its schema should be defined in a single source of truth (likely an extension of `content.json`) so the Registry, Snapshots, and Persistence layers are auto-generated.

## 4. The Hidden Cost of `UserData`
>
> **Architect's Solution:** Use `mlua::UserData` for hot-path components.

- **Adversarial Critique:** `UserData` in `mlua` involves `RefCell`-like overhead for every access. While faster than JSON, it is not "free."
- **Requirement:** We must implement a **Benchmark Suite** specifically for FFI component access. We need to know exactly how many `UserData` accesses we can afford per turn before we start dropping frames.

## 5. Script Error Handling: "Silent Failures"
>
> **Architect's Solution:** Script errors surface as logged warnings and fall back to default behaviour.

- **Adversarial Critique:** "Silent fallback" is a debugging nightmare. If a boss's "Phase 2" transition script fails, and it silently falls back to "Phase 1" behavior, the player encounters a bug that the developer might never see in logs unless they are looking for it.
- **Requirement:** We need a **Lua Panic UI** or a **Visual Debugger**. If a script fails during development, it should be impossible to miss. In production, we need a "Safe State" recovery that doesn't just "do nothing" but puts the entity into a predictable fallback state (e.g., a simple "Wait" or "Default Melee" action).

---

### Revised Verdict (Round 2)

**Status:** **PROCEED WITH CAUTION.**
The Command Buffer is the correct path, but the "Monolithic Snapshot" and "Manual Schema Registration" are traps that will lead to a bloated, unmaintainable middle-layer.

**Next Steps:** Implement a prototype of the **Command Buffer** for a single item (e.g., a "Scroll of Chaos" that requires complex logic) and profile the **Lazy Snapshot** overhead before committing to a full AI overhaul.
