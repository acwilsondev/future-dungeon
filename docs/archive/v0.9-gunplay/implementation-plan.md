# Implementation Plan: v0.9-gunplay

**Status:** Draft
**Epic:** v0.9-gunplay

This document outlines the technical changes required to implement the Gunplay
system as defined in `gunplay.md` and `aegis.md`. Concrete items, enemies, and
playtest scenarios live in `content.md` (the canonical content spec). This plan
extends (not replaces) the existing ranged weapon pipeline in
`src/app/items_use_ranged.rs`.

The work divides into four vertical slices:

1. **Aegis** — a new shield stock layered in front of HP.
2. **Cover** — a partial-cover tile feature that raises Dodge DC.
3. **Overheating** — replace ammo-count bookkeeping with a per-weapon Heat meter.
4. **Weapon Modifiers** — Burst, Scatter, Heavy, Shredding, Tachyonic, Melee,
   Elemental — implemented as data-driven tags on `RangedWeapon`.

Each slice is independently shippable and is gated behind its own tests.

---

## 1. Aegis

### 1.1 Components (`src/components.rs`)

- Add `Aegis { current: i32, max: i32 }` — equipped on any entity that has
  `CombatStats`. Absence of the component means no aegis (HP-only). Persists
  across save/load (mirror the snapshot handling of `CombatStats` in
  `src/app/snapshot.rs` and `serialization.rs`).
- Add `AegisDrought { duration: u32 }` — debuff that blocks aegis regen.
- Add `AegisBoost { magnitude: i32, duration: u32 }` — buff granting temporary
  aegis (stacking rules: additive with a single instance; re-application
  refreshes duration and takes max of magnitudes).

### 1.2 Damage Layer (`src/app/combat.rs`)

**Mental model: Aegis is a force field.** Projectiles and spells interact
with it from outside. Things already past the field (melee attackers in
contact, stepped-on traps) never interact with it. Things already inside the
body (DOTs) also skip AV — the armor has already been bypassed.

**AV applies to Health only, never to Aegis.** AV still gates HP, but it
stops doing so by being baked into the damage number inside `resolve_attack`
(line 142: `damage = (weapon_roll + attr_mod + power_bonus -
target_av).max(1)`). Instead, AV is applied by the damage-application layer,
and only on the HP-bound portion.

#### Refactor `resolve_attack`

Remove the `- target_av` term and the `target_av` clamp from `resolve_attack`.
Keep `target_av` on `AttackResult` for logging, but the returned `damage`
becomes *raw* pre-mitigation damage. Tests that depend on the current behavior
(`test_zero_defense_damage_at_least_one`, `test_overkill_damage_does_not_panic`)
need to be updated — minimum-1 clamp moves to the final HP-application step.

#### New entry point

`apply_damage(target, raw: i32, DamageRoute)` replaces the direct
`stats.hp -= res.damage` writes. Three routes, distinguished by how far
"inside" the target the damage originates:

```rust
enum DamageRoute {
    Projectile, // Outside the shield.  Aegis -> (AV -> HP).
                // Ranged attacks, thrown items, damaging spells.
                // Tachyonic/Shredding modify this path.
    Contact,    // Past the shield, still outside the body.  (AV -> HP).
                // Melee bumps, stepped-on traps, environmental hazards.
    Systemic,   // Inside the body.  HP only, no mitigation.
                // Status-effect DOTs (poison, future bleed/burn/etc.).
}
```

Reroute all existing callers:

| Caller | Route |
| --- | --- |
| `apply_attack_result` melee branch (`combat.rs`) | `Contact` |
| `apply_attack_result` ranged branch (`combat.rs`) | `Projectile` |
| `handle_aoe_effect` in `items_use_ranged.rs` (thrown/fired AOE) | `Projectile` |
| Spell damage opcodes in `casting.rs` | `Projectile` |
| Poison tick in `turn_tick::apply_status_effects` | `Systemic` |
| Future status-effect DOTs (bleed, burn, etc.) | `Systemic` |
| Trap damage in `player_move.rs` | `Contact` |

**Rationale for traps as Contact**: the player steps onto the trap, so the
Aegis field has already been breached, but physical armor (boots, greaves,
etc.) still matters. This gives armor a role outside of combat and matches
the force-field framing.

#### Resolution order (the important part)

For **Projectile** on a target with Aegis:

1. `to_aegis = min(raw, Aegis.current)`; `Aegis.current -= to_aegis`.
2. `overflow = raw - to_aegis`.
3. If `overflow > 0`: `hp_damage = max(1, overflow - target_av)`;
   `stats.hp -= hp_damage`.

For **Contact** (melee, traps) and for **Projectile** on an aegis-less or
0-aegis target:

- `hp_damage = max(1, raw - target_av)`; `stats.hp -= hp_damage`.

For **Systemic**:

- `stats.hp -= raw` with no clamp, no mitigation.

Notes:

- AV is applied *once*, only to HP-bound damage. A Projectile hit fully
  absorbed by Aegis never touches AV and never triggers the `max(1)` floor.
- The minimum-1-damage floor now lives at the HP step, so a fully
  aegis-absorbed hit can leave HP untouched entirely. Intended.
- `get_target_av` (combat.rs:259) stays as-is; it is now called only from
  the HP branch of `apply_damage`.

#### Aegis drought on damage

- Any hit that reduces `Aegis.current` below `Aegis.max` applies
  `AegisDrought(5)`.
- Any hit that *depletes* Aegis (current → 0) applies `AegisDrought(10)`.
- Re-application takes the max of the existing and new duration (never
  shortens an active drought).

### 1.3 Regen (`src/app/turn_tick.rs`)

- New method `regen_aegis` called from `on_turn_tick`. For every entity with
  `Aegis` and without `AegisDrought`, increment `Aegis.current` by 1 clamped to
  `Aegis.max`.
- Extend `apply_status_effects` to tick `AegisDrought` and `AegisBoost`
  durations and remove expired entries. When `AegisBoost` expires, subtract
  its magnitude from `Aegis.current` (clamped ≥0) *and* `Aegis.max`.

### 1.4 Player Initialization (`src/spawner.rs`)

- `spawn_player` gains `Aegis { current: 5, max: 5 }` by default. (Tuning value;
  easy to revisit.)
- Some content-defined monsters (mech-flavored enemies, bosses) may declare
  aegis in `content.json` via a new `aegis: Option<i32>` on `RawMonster`.

### 1.5 UI (`src/renderer.rs`)

Modify `draw_sidebar` per `aegis.md` §"Aegis GUI":

- Remove the fixed 3-line HP gauge; replace with a combined `HP+Aegis` gauge
  whose total length = `hp_max + aegis_max`. Draw Aegis segment in cyan
  (`Color::Rgb(0, 220, 220)`), HP segment in the current color ramp
  (green/yellow/red by hp%).
- If Aegis is absent or max==0, render exactly the current HP gauge (no
  regressions for non-aegis entities).
- Add status line for `AegisDrought` (dulled cyan) and `AegisBoost` (bright
  cyan) in the status pane, mirroring `ManaDrought` handling.

### 1.6 Tests

- `Projectile` hit on aegis-having target reduces Aegis before HP.
- **AV does not apply to Aegis**: 8 raw `Projectile` damage against a
  target with 5 Aegis and AV 3 → Aegis drops 5, overflow 3 hits AV → HP
  loses 0 floored to 1.
- **AV applies to HP-bound overflow exactly once**: 10 raw `Projectile`
  damage against 3 Aegis / AV 2 → Aegis drops 3, overflow 7, HP loses 5.
- **`Contact` applies AV, bypasses Aegis**: 6 raw melee damage against
  5 Aegis / AV 2 → Aegis unchanged, HP loses 4.
- **Trap damage applies AV, bypasses Aegis**: 5 raw trap damage against
  5 Aegis / AV 2 → Aegis unchanged, HP loses 3.
- Fully aegis-absorbed Projectile hit does not trigger the min-1 damage
  floor (HP unchanged).
- Partial aegis damage → `AegisDrought(5)` applied.
- Full aegis depletion → `AegisDrought(10)` applied (and overrides a smaller
  active drought; does not shorten a larger one).
- `AegisDrought` prevents regen; expiry allows regen.
- **`Systemic` bypasses both Aegis and AV**: 2 poison damage against a
  target with 5 Aegis / AV 3 deducts 2 from HP, leaves Aegis untouched.
- Poison tick routes through `apply_damage` as `Systemic` — verify the
  updated `turn_tick::apply_status_effects` tests still hold after the
  refactor.
- Snapshot round-trip preserves `Aegis` component.
- Existing `test_zero_defense_damage_at_least_one` and
  `test_overkill_damage_does_not_panic` updated: `AttackResult.damage` is
  now *raw* damage, the min-1 clamp lives at HP application.

---

## 2. Cover

### 2.1 Model

Full Cover already exists implicitly: walls block both LOS and movement, so
`items_use_ranged::fire_targeting_item` already terminates the projectile at
the wall tile (`self.map.blocked[idx]` check at line ~237).

Partial Cover is the new piece. It is a **tile-adjacent feature** that
does not block movement or LOS but raises the target's Dodge DC by +2 against
ranged attacks coming from the opposite side.

### 2.2 Components (`src/components.rs`)

- Add `PartialCover` (zero-sized) — attached to a map entity at a position,
  rendered as `.` in a distinct color (per `gunplay.md` §1, "Low debris `.`").
  Alternative considered: a `TileType::Rubble` variant; rejected to avoid
  rippling through `map_builder.rs`, FOV, `BaseMap`, renderer biome matrices,
  and save formats. An ECS entity tag is the lighter touch.

### 2.3 Dodge DC Calculation (`src/app/combat.rs`)

Extend `resolve_attack` (it currently computes `dodge_dc = 10 + target_dex_mod`
at line 114) to add `+2` when all of:

1. `is_ranged == true`
2. Target position has a `PartialCover` entity, OR the tile *between* attacker
   and target along the Bresenham line (immediately adjacent to target) has one.

The "opposite side" rule from `gunplay.md` §1 is implemented as: walk the line
from attacker to target; if any `PartialCover` is on the penultimate step
(the tile the shot passes through just before reaching the target), apply the
bonus.

### 2.4 Spawner (`src/spawner.rs`)

- `spawn_partial_cover(world, x, y)` — glyph `.`, color tan
  (`Color::Rgb(150, 120, 80)`), `RenderOrder::Map`, `PartialCover`,
  `Name("Debris")`.

### 2.5 Level Generation (`src/app/level_gen.rs`)

- Room feature placer: with low probability (e.g., 1 per 3 rooms on average),
  scatter 1–3 partial cover tiles along the room interior on non-edge floor
  tiles. Cluster near walls/corners to read as real terrain.

### 2.6 Tests

- Firing at a target behind partial cover: roll vs. DC that includes +2.
- Firing at a target with partial cover on the *far* side of them (not between
  attacker and target): no bonus.
- Partial cover does not block LOS (wall check at `items_use_ranged.rs:237`
  remains intact; partial cover tiles are not in `map.blocked`).
- Melee bump attack ignores the cover bonus entirely.

---

## 3. Overheating (Heat Meter)

### 3.1 Components (`src/components.rs`)

- Add `HeatMeter { current: u32, capacity: u32, venting: u32 }` to
  `RangedWeapon`-tagged items. `venting > 0` means the weapon is offline for
  `venting` more turns.
- Add a `heat_per_shot: u32` field (default 1) and an `efficient_cooldown:
  bool` field (per `gunplay.md` §2, "Efficient Cooldown" modifier) to
  `RangedWeapon`. These are data-driven via `content.json`
  (`RawItem::ranged_weapon`, which is currently a 3-tuple and will need to
  grow into a struct — see §5.1 below).

### 3.2 Fire Path (`src/app/items_use_ranged.rs`)

- At the top of `fire_targeting_item`, check the weapon's `HeatMeter.venting`.
  If >0, log `"The {weapon} is venting heat."` and return without advancing
  the turn — i.e., do not set `state = RunState::MonsterTurn`. This lets the
  player retry with a different action; it does not trap them. (Open question
  for design review: should an attempt to fire while venting still cost a
  turn? Default answer: yes, it costs a turn. Leave the turn advance in.)
- On successful fire:
  - `heat.current += weapon.heat_per_shot * shots_this_fire` (shots > 1 for
    Burst — see §4.1).
  - If `heat.current >= heat.capacity`, set `heat.venting` to `VENT_DURATION`
    (default 3; modifier `efficient_cooldown` reduces to 1) and reset
    `heat.current = 0`. Log `"The {weapon} vents superheated gas!"` and emit a
    visual effect (steam puff via `VisualEffect::Flash`).

### 3.3 Cooldown (`src/app/turn_tick.rs`)

- Per turn: for every ranged weapon entity with `HeatMeter`, if `venting > 0`,
  decrement; else decrement `current` by 1 (floored at 0). This gives passive
  cooldown while not firing.

### 3.4 Ammo Deprecation

`gunplay.md` §4 replaces ammo-counting with heat *for firearms*. Bows and
crossbows remain traditional ammo consumers. Introduce a discriminant on
`RangedWeapon`:

```rust
pub enum WeaponPowerSource {
    Ammo,       // existing: Ammunition item in backpack
    HeavyAmmo,  // gunplay.md §2: stackable fungible heavy rounds
    Heat,       // gunplay.md §4: HeatMeter
}
```

`consume_ammo` in `items_use_ranged.rs:24` branches on power source:

- `Ammo`: existing Ammunition-in-backpack consumption.
- `HeavyAmmo`: despawn one `HeavyAmmo` item; if none, abort with message.
- `Heat`: accumulate into `HeatMeter`; no inventory consumption.

### 3.5 UI (`src/renderer.rs`)

- Sidebar: when the equipped main-hand has `HeatMeter`, draw a thin red gauge
  labeled `Heat` below the HP/Aegis bar. When `venting`, show the remaining
  vent turns in orange.

### 3.6 Tests

- Firing a Heat weapon increments `current` by `heat_per_shot`.
- Hitting capacity triggers a vent: `venting` set, `current` reset.
- Venting weapon cannot fire; `state` behavior matches design decision in §3.2.
- Passive cooldown: a non-firing turn decrements `current`.
- `efficient_cooldown` shortens vent duration.

---

## 4. Weapon Modifiers

All modifiers are additive tags on `RangedWeapon`. None of them are
individually a new RunState; they re-shape the existing fire flow.

### 4.1 Burst

- Field: `burst_count: u32` (default 1).
- In `fire_targeting_item`, if `burst_count > 1`, call `handle_direct_damage`
  in a loop. The first shot uses the computed `disadvantage_count`; each
  subsequent shot adds +1 disadvantage. Existing off-hand ranged proc
  (`items_use_ranged.rs:297`) is unaffected (still one proc, not per-burst).
- Heat: burst contributes `heat_per_shot * burst_count`.
- Test: verify 3 attack rolls for a 3-burst, second-roll disadvantage=1,
  third-roll disadvantage=2.

### 4.2 Scatter

- Field: `scatter: bool`.
- Replaces the range-increment disadvantage formula. In the disadvantage
  computation (currently `items_use_ranged.rs:260-266`), if `scatter` is set:
  - `disadvantage_count = 0` regardless of distance.
  - Instead, drop damage die by one step per range increment over the base
    range. Step ladder (defined once in `src/app/combat.rs`):
    `d12 → d10 → d8 → d6 → d4 → d2` (min d2). Apply in
    `resolve_attack` when the weapon has the `scatter` flag by overriding the
    die type before rolling.
- Test: scatter weapon at 3× range increment rolls base_die stepped down by 3.

### 4.3 Heavy

- See §3.4. `WeaponPowerSource::HeavyAmmo` — consumes from a fungible
  `HeavyAmmo` item stack. Add an `ItemStack { count: u32 }` component to
  support fungibility; `consume_ammo` decrements count and despawns at 0.
  (Existing `Ammunition` remains 1-per-entity for now — migrating arrows to
  the stack model is out of scope; track as tech debt.)

### 4.4 Shredding

Shredding applies a stackable `Shredded` debuff instead of mutating the
target's equipment. Each stack reduces the target's effective AV by 1 (floored
at 0). Stacks **decay one at a time** — every 5 turns without re-application,
a single stack falls off. Any new application adds a stack and resets the
decay timer, so a freshly-re-shredded target loses stacks slower than an
untouched one.

Example: after 5 consecutive Shredding hits, target has `Shredded(5)`. If
never shredded again: T+5 → Shredded(4), T+10 → Shredded(3), ..., T+25 →
removed. A single re-application during that window resets the countdown to
5 and bumps the stacks.

This sidesteps the awkwardness of mutating `Armor.defense_bonus` on a shared
item entity (multiple monsters could in principle share armor types in
future; and "shredding a monster's natural defense" is confusing when no
armor item exists).

#### 4.4.1 Stackable status effects — prerequisite work

The existing status effects (`Poison`, `Confusion`, `Mired`, `Armored`,
`Strength`) are all single-instance: `apply_attack_result` in
`combat.rs:342-367` guards with `if self.world.get::<&Poison>(target).is_err()`
before inserting, and re-application refreshes rather than stacks. We don't
yet have a stacking model.

Introduce the minimum viable stacking:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shredded {
    pub stacks: u32,
    pub decay_timer: u32,  // counts down to 0, then sheds one stack and resets
}
```

Helper on `App` (or a free function in a new
`src/app/status_effects.rs` — likely worth its own module once we're mixing
stacking and non-stacking):

```rust
const SHREDDED_DECAY_INTERVAL: u32 = 5;
const SHREDDED_CAP: u32 = 10;

fn apply_shredded(&mut self, target: Entity, new_stacks: u32) {
    if let Ok(mut s) = self.world.get::<&mut Shredded>(target) {
        s.stacks = (s.stacks + new_stacks).min(SHREDDED_CAP);
        s.decay_timer = SHREDDED_DECAY_INTERVAL;  // reset on reapplication
    } else {
        let _ = self.world.insert_one(target, Shredded {
            stacks: new_stacks.min(SHREDDED_CAP),
            decay_timer: SHREDDED_DECAY_INTERVAL,
        });
    }
}
```

- Stacks add; `decay_timer` *always* resets to `SHREDDED_DECAY_INTERVAL` on
  any re-application, even if the new application wouldn't raise stacks
  (e.g. already at cap). This is the core "re-hit keeps the pressure on"
  behavior.
- Soft cap of 10 prevents pathological accumulation.
- This is the template for any future stackable effect that needs graceful
  decay (bleed, burn, etc.) — keep the signature uniform:
  `(stacks)`, with the decay interval as a module constant.

#### 4.4.2 AV integration (`src/app/combat.rs`)

`get_target_av` (combat.rs:259) already sums equipment armor + `CombatStats.defense`.
Extend it to subtract `Shredded.stacks` with a floor of 0:

```rust
pub fn get_target_av(&self, entity: Entity) -> i32 {
    let mut def = /* existing sum */;
    if let Ok(s) = self.world.get::<&Shredded>(entity) {
        def = (def - s.stacks as i32).max(0);
    }
    def
}
```

Because AV is only consulted on the HP branch of `apply_damage` (§1.2), this
has no effect while Aegis is soaking — Shredding continues to "do nothing
until the shield is down," which matches the force-field mental model.

#### 4.4.3 Shredding hit behavior

- Field on `RangedWeapon`: `shredding: bool`.
- In `apply_damage`, a `Projectile` hit from a Shredding weapon calls
  `apply_shredded(target, 1)` *whenever the hit lands and deals any raw
  damage*, regardless of whether the damage was fully absorbed by Aegis. (The
  "armor isn't being shredded yet while the shield's up" intuition is
  preserved by the AV integration: the stacks exist but don't take effect
  until Aegis breaks.)
- Monsters with natural AV from `CombatStats.defense` are also affected —
  Shredded reduces total AV, not armor-item AV specifically. A monster with
  no equipped armor but `defense: 3` still has its effective defense chewed
  to 0 after 3 stacks.

#### 4.4.4 Tick (`src/app/turn_tick.rs`)

Extend `apply_status_effects` with a graceful-decay pattern (new shape — not
shared with `Mired`/`Armored`, which expire all-at-once):

```rust
for (id, mut s) in self.world.query::<&mut Shredded>().iter() {
    if s.decay_timer > 0 {
        s.decay_timer -= 1;
    }
    if s.decay_timer == 0 {
        s.stacks = s.stacks.saturating_sub(1);
        if s.stacks == 0 {
            to_remove_shredded.push(id);
        } else {
            s.decay_timer = SHREDDED_DECAY_INTERVAL;
        }
    }
}
```

- A stack drops every `SHREDDED_DECAY_INTERVAL` turns of non-reapplication.
- When `stacks` hits 0, the component is removed entirely.
- No saving throw — the effect is a physical state, not a condition to resist.

#### 4.4.5 UI (`src/renderer.rs`)

- Sidebar status line for the *player*: `Shredded x{stacks} ({decay_timer})`
  in a muted steel color. The decay number ticks down visibly; it's useful
  feedback on whether to break contact or press the attack.
- When rendering enemies, no UI change — stacks are visible indirectly by
  the changed damage output. (A future HUD pass could show it on hover.)

#### 4.4.6 Tests

- Single Shredding hit applies `Shredded { stacks: 1, decay_timer: 5 }`.
- Second Shredding hit before decay: stacks → 2, timer reset to 5.
- AV reduction: target with AV 4 and `Shredded(2)` has effective AV 2.
- Floor at 0: target with AV 1 and `Shredded(5)` has effective AV 0 (never
  negative).
- Soft cap: 15 consecutive shredding hits yield `stacks: 10`, not 15.
- **Graceful decay**: start at `Shredded(5)`. After 5 turns with no
  re-application → `Shredded(4)`. After 5 more → `Shredded(3)`. Component
  is fully removed exactly 25 turns after the last application.
- **Decay reset on re-application**: `Shredded(3)` at `decay_timer: 1`
  (one turn from shedding) — a new Shredding hit takes it to
  `Shredded(4)` with `decay_timer: 5`, not `Shredded(2)` with
  `decay_timer: 5`.
- **Reset even at cap**: `Shredded(10)` at `decay_timer: 1` — a new hit
  keeps stacks at 10 and resets timer to 5.
- Stacks apply even when the triggering hit is fully absorbed by Aegis
  (design call; easy to flip).
- Snapshot round-trip preserves `Shredded` including `decay_timer`.

### 4.5 Tachyonic

- Field: `tachyonic: bool`.
- In `apply_damage` with route `Projectile`, if the target has
  `Aegis.current > 0` and the weapon is Tachyonic: the Aegis soak step
  consumes `min(raw * 2, Aegis.current)` from Aegis. Overflow to HP is
  computed from the *original* raw amount minus the portion of raw that
  Aegis "paid for".
- Simpler equivalent formulation (preferred in code for clarity):
  1. `aegis_consumed = min(raw * 2, Aegis.current)`
  2. `raw_absorbed = ceil(aegis_consumed / 2)` (the raw that Aegis ate)
  3. `overflow = raw - raw_absorbed`
  4. HP branch applies AV to `overflow` as normal.
- Test cases:
  - Tachyonic hit for 3 raw against 10 Aegis: Aegis drops 6, overflow 0.
  - Tachyonic hit for 5 raw against 6 Aegis: Aegis drops 6 (clamped),
    `raw_absorbed = 3`, overflow = 2, applied through AV to HP.
  - Tachyonic hit for 4 raw against 0 Aegis: behaves as a normal ranged hit
    (AV → HP, no doubling).

### 4.6 Melee (improved bump)

- Field: `melee_profile: Option<Weapon>` — when the player bump-attacks with
  this ranged weapon equipped in main hand, `combat::resolve_attack` already
  uses the item's `Weapon` component in the non-`is_ranged` branch
  (lines 76-82). Extend content loading so a `RangedWeapon` can carry its own
  melee die/bonus that overrides the default `Weapon` if both are present.
  Most firearms keep a light default (e.g., 1d4 bludgeoning pistol-whip); the
  "Melee" modifier weapons get a better profile.

### 4.7 Elemental

- Field: `element: Option<DamageType>`.
- In `resolve_attack`, tag the attack result with the element. Resistance /
  weakness systems don't exist yet and are out of scope; for now, elemental
  hits produce a themed `VisualEffect::Flash` color (Fire=orange,
  Poison=green, etc.) and a log tag `"(Fire)"`. Lays the groundwork for
  resistance in a later epic.

---

## 5. Content & Data

### 5.1 `RawItem` schema (`src/content.rs`)

Current `ranged_weapon: Option<(i32, i32, i32)>` (range, increment, damage
bonus) is positional and can't express the new fields. Migrate to a struct:

```rust
pub struct RawRangedWeapon {
    pub range: i32,
    pub range_increment: i32,
    pub damage_bonus: i32,
    #[serde(default)] pub power_source: Option<String>, // "ammo"|"heavy"|"heat"
    #[serde(default)] pub heat_capacity: Option<u32>,
    #[serde(default)] pub heat_per_shot: Option<u32>,
    #[serde(default)] pub efficient_cooldown: bool,
    #[serde(default)] pub burst_count: Option<u32>,
    #[serde(default)] pub scatter: bool,
    #[serde(default)] pub shredding: bool,
    #[serde(default)] pub tachyonic: bool,
    #[serde(default)] pub element: Option<String>, // parses via existing parse_damage_type
}
```

Keep the old tuple form loadable via a custom `Deserialize` adapter so
`content.json` and existing tests don't break in the same PR.

### 5.2 New items in `content.json`

See `content.md` §1–§2 for the canonical list with stat tables, spawn tables,
and design notes. At a glance, the seed set covers every modifier at least
once:

| Item | Modifiers | Power |
| --- | --- | --- |
| Service Pistol | — (baseline) | Heat |
| Scattergun | Scatter | Heat |
| Carbine | Burst 3 | Heat |
| Heavy Rifle | Shredding | HeavyAmmo |
| Tachyon Lance | Tachyonic, Efficient Cooldown | Heat |
| Phoenix Repeater | Elemental: Fire | Heat |
| Monk's Crook | Improved Melee | Heat |
| Heavy Ammo | (stackable fungible) | — |

### 5.3 `RawMonster`

- Optional `aegis: Option<i32>` for mech-flavored enemies.
- Seed aegis-having enemies per `content.md` §3: Rampart Sentry (aegis 6),
  Vault Drone (aegis 10, glass-shield), Temple Bastion (aegis 12, armored
  miniboss).

### 5.4 Starting Loadouts (`src/app/actions.rs`)

- Fighter (`actions.rs:154`): append `"Carbine"` to `starting_items` and equip
  it (main hand). Replaces no existing item — sits alongside Longsword so the
  swap-during-vent pattern works from turn one.
- Nihil Initiate + Solari Initiate (shared path at `actions.rs:236`): append
  `"Service Pistol"` to `starting_items` and equip it. Gives casters a
  reliable non-mana ranged option from turn one.
- Update the corresponding tests: the Fighter loadout assertions in
  `level_gen.rs:341-347` and the Initiate class tests at `actions.rs:487`
  / `actions.rs:513` need to expect the new starting weapon in the
  backpack/equipment.

### 5.5 Partial Cover Placement

Level-gen tuning per `content.md` §4: bias placement toward rooms with
Tactical-personality enemies; rarer-but-paired with Rampart Sentries in the
Vaults branch; never on stairs or choke points.

---

## 6. Actions & UI

### 6.1 Existing `Action::Target` (`F` key)

The current `f` key flow (`input.rs:27` → `trigger_ranged_targeting` at
`items_use_ranged.rs:336`) is reused unchanged for the common case. No new
action is required for the core gunplay loop.

### 6.2 Sidebar

Covered in §1.5 (Aegis bar) and §3.5 (Heat gauge).

### 6.3 Inventory Rendering (`render_inventory` in `renderer.rs`)

Extend weapon description lines to include modifier tags:
`Service Pistol (Heat 0/6, Burst 1)`, `Scattergun (Heat 0/4, Scatter)`, etc.

---

## 7. Save/Load

- Add `Aegis`, `AegisDrought`, `AegisBoost`, `HeatMeter`, `PartialCover`,
  `ItemStack` to `EntitySnapshot` in `src/app/snapshot.rs` and their
  serialization hooks in `src/app/serialization.rs`. Mirror the existing
  `ManaPool` / `Mired` / `Armored` patterns.
- Migration of existing savegames: on load, absent components default to the
  current behavior — no aegis, no heat, etc. `persistence.rs::load_game`
  requires no special-casing.

---

## 7b. Mana Regen Tuning

Motivated by the gunplay epic itself: now that every class starts with a
firearm (Fighter/Carbine, Initiates/Service Pistol per §5.4), the "I cast a
spell every turn because it's free" pattern needs to change. Spells should
feel special; even Initiates should default to their sidearm and cast when
the moment matters.

### 7b.1 Change

In `src/app/mana_regen.rs`, `tick_mana_regen` currently grants +1 pip to the
greatest-deficit color every call (i.e., every turn). Slow this to **1 pip
per 5 turns**.

Implementation: add a `regen_cooldown: u32` counter to `ManaPool` (or a
sibling singleton on the entity — `ManaPool` is simpler). Each tick,
decrement; only when it hits 0 do the existing regen logic, then reset to
`MANA_REGEN_INTERVAL = 5`.

```rust
pub struct ManaPool {
    pub current_orange: u32,
    pub max_orange: u32,
    pub current_purple: u32,
    pub max_purple: u32,
    pub regen_cooldown: u32,  // turns until next pip
}
```

Notes:

- `ManaDrought` logic is unchanged: drought still freezes regen. When drought
  ends, the `regen_cooldown` resumes from wherever it was (don't reset it on
  drought end — the player shouldn't be able to game drought for a free
  pip).
- Starting value on spawn: `regen_cooldown: MANA_REGEN_INTERVAL` so the
  first pip comes after 5 turns, not on turn 1.
- Shrines, tomes, and potions that grant mana directly (per
  `src/app/shrine.rs`, `tome.rs`, `actions_alchemy.rs`) do *not* interact
  with the cooldown — they grant mana immediately as today.

### 7b.2 Snapshot / serialization

Add `regen_cooldown` to the `ManaPool` snapshot in `src/app/snapshot.rs` and
`serialization.rs`. Default to `MANA_REGEN_INTERVAL` on load when absent so
pre-v0.9 saves get a 5-turn grace period rather than a free pip on the
first tick.

### 7b.3 Tests (`src/app/mana_regen.rs`)

Update the existing tests — all of them currently assert regen happens on a
single `tick_mana_regen()` call, which is no longer true.

- `test_regen_fills_deficit`: must call `tick_mana_regen()` 5× to see +1.
- `test_regen_chooses_greatest_deficit`: 5× tick, then assert.
- `test_regen_skipped_during_drought`: 5× tick with drought active; assert
  no pip and cooldown unchanged (or decremented only after drought — pick
  the "frozen during drought" variant).
- `test_drought_expires`: unchanged.
- `test_regen_no_deficit`: 5× tick, assert pool full still.
- **New** `test_regen_rate_is_one_per_five_turns`: empty pool, 4 ticks →
  no change; 5th tick → +1 pip.
- **New** `test_cooldown_persists_across_drought`: cooldown at 2, apply
  drought, tick 10×, drought expires, one more tick → pip lands (cooldown
  was frozen, not reset).
- **New** Snapshot round-trip preserves `regen_cooldown`.

### 7b.4 UI (optional)

Consider surfacing the cooldown in the status pane — e.g., a small `Mana↑`
countdown beside the mana bar. Not required for v0.9; skip if it crowds the
sidebar.

---

## 8. Validation Plan

### 8.1 Unit Tests

Per subsystem as listed in §1.6, §2.6, §3.6, §4.1–§4.7. All live in the
existing per-file `#[cfg(test)] mod tests` blocks.

### 8.2 Integration Tests

- Full engagement: player fires Tachyon Lance at an aegis-having monster →
  aegis drops 2× fast, drought applied, next turn no regen, turn after
  regen resumes when drought expires.
- Burst + Scatter interaction: a hypothetical weapon with both is rejected at
  content load time (they're incoherent; validate and bail in
  `Content::validate`).
- Heat vent locks out firing, eventually resumes.
- Partial cover on the line of fire adds +2 DC; removed behind-target case
  does not.

### 8.3 Manual QA

Because `CLAUDE.md` notes that type checks don't verify feature correctness:
play a run from D:1 to D:3 using each new weapon at least once, with at least
one run confirming the Aegis GUI renders correctly on small terminals
(80×24). Run the four scenarios in `content.md` §5 ("The Bastion Dance",
"Cover Fire", "Heat Budget", "Aegis Drought") as targeted acceptance checks.

---

## 9. Rollout Order

Recommend landing in this order so each PR is independently testable:

1. Aegis (§1) — self-contained, touches combat routing once.
2. Partial Cover (§2) — self-contained, touches `resolve_attack` and level gen.
3. Heat + `WeaponPowerSource` refactor (§3 + §3.4) — requires §5.1 schema
   migration.
4. Weapon modifiers (§4) — builds on the new schema.
5. Content seeding (§5.2) + starting loadouts (§5.4) — after the data model
   is final.
6. Mana regen tuning (§7b) — lands alongside or after content seeding so the
   balance shift arrives with the weapons that justify it.

Each step should leave `make lint` and `make harden` green.
