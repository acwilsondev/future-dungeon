# Gunplay Content

**Status:** Draft
**Epic:** v0.9-gunplay

This document seeds concrete content that exercises every feature in
`gunplay.md`, `aegis.md`, and `implementation-plan.md`. Stats are tuning
targets, not commitments — numbers are expected to shift during playtesting.

The goal is to touch each modifier at least once (Burst, Scatter, Heavy,
Shredding, Tachyonic, Melee, Elemental, Efficient Cooldown), each power
source (Ammo, HeavyAmmo, Heat), and both defensive pieces (Aegis, Partial
Cover).

---

## 1. Firearms

### 1.1 Service Pistol

Baseline heat weapon. Nothing fancy; it's the tutorial firearm.

| Field | Value |
| --- | --- |
| Glyph / Color | `)` / steel grey |
| Slot | AnyHand (1H) |
| Range / Increment | 6 / 4 |
| Damage | 1d6 + 1 |
| Power Source | Heat |
| Heat Capacity | 6 |
| Heat / Shot | 1 |
| Modifiers | — |
| Spawn | D:1–D:5, chance 0.3, price 30g |

Design note: capacity 6 lets the player fire six shots before a vent — long
enough that Heat doesn't feel like a nerfed ammo system, short enough that
the Fighter's "swap to longsword while cooling" play pattern (gunplay.md §4)
is rewarding.

**Starting weapon for Nihil Initiate and Solari Initiate** — added to the
shared starting inventory in `src/app/actions.rs:236` alongside Leather
Armor / Torch / Dagger / Health Potion. Gives the caster classes a reliable
non-mana ranged option from turn one.

### 1.2 Scattergun

Classic scatter design. Punishes close quarters, falls off sharply at range.

| Field | Value |
| --- | --- |
| Glyph / Color | `}` / burnt copper |
| Slot | AnyHand, two-handed |
| Range / Increment | 3 / 3 |
| Damage | 2d6 |
| Power Source | Heat |
| Heat Capacity | 4 |
| Heat / Shot | 2 |
| Modifiers | **Scatter** |
| Spawn | D:2–D:8, chance 0.15, price 80g |

Scatter behavior: no disadvantage for range, but damage die steps down per
increment (2d6 → 2d4 → 2d2). At 4+ increments the weapon is essentially
noisemaking; this is the intended pressure to close.

### 1.3 Carbine

The "plink plink plink" weapon. Three shots per trigger pull, stacking
disadvantage on shots 2 and 3.

| Field | Value |
| --- | --- |
| Glyph / Color | `}` / gunmetal |
| Slot | MainHand, two-handed |
| Range / Increment | 8 / 8 |
| Damage | 1d4 + 1 |
| Power Source | Heat |
| Heat Capacity | 9 |
| Heat / Shot | 1 (burst counts as 3) |
| Modifiers | **Burst 3** |
| Spawn | D:3–D:10, chance 0.2, price 110g |

A full burst consumes 3 heat. At capacity 9 that's three full bursts before
a vent — a natural rhythm of three engagements.

**Starting weapon for Fighter** — added to the Fighter starting inventory in
`src/app/actions.rs:154` alongside Chainmail / Shield / Torch / Longsword /
Health Potion. Pairs with the longsword-swap-during-vent pattern in
`gunplay.md` §4.

### 1.4 Heavy Rifle ("Breaker")

High-damage, armor-chewing, slow. Uses Heavy Ammo instead of heat.

| Field | Value |
| --- | --- |
| Glyph / Color | `}` / dull iron |
| Slot | MainHand, two-handed |
| Range / Increment | 12 / 10 |
| Damage | 2d8 + 2 |
| Power Source | HeavyAmmo |
| Modifiers | **Shredding** |
| Spawn | D:4–D:12, chance 0.1, price 220g |

Each hit lands a `Shredded(1)` stack (10-stack cap, 5-turn decay). On a
tough armored target, three quick hits leave it with `Shredded(3)` for
~25 turns — a meaningful window for the party to follow up. Heavy Ammo is
the cost: miss your shots and you're out.

### 1.5 Tachyon Lance

Anti-shield specialist. Mediocre damage, devastating against Aegis-having
foes.

| Field | Value |
| --- | --- |
| Glyph / Color | `}` / cyan |
| Slot | MainHand, two-handed |
| Range / Increment | 10 / 8 |
| Damage | 1d10 |
| Power Source | Heat |
| Heat Capacity | 4 |
| Heat / Shot | 1 |
| Modifiers | **Tachyonic**, **Efficient Cooldown** |
| Spawn | D:6–D:15, chance 0.08, price 350g |

Against a 6-aegis target: a single 5-raw Lance hit breaks the shield
(`min(5*2, 6) = 6` aegis consumed, overflow 2 to HP). Efficient Cooldown
caps vent at 1 turn — the Lance recovers in time to chase through the
overflow window.

### 1.6 Phoenix Repeater

The Elemental example. Fire-tagged shots that synergize with any future
burning/ignite system.

| Field | Value |
| --- | --- |
| Glyph / Color | `)` / orange-red |
| Slot | AnyHand (1H) |
| Range / Increment | 6 / 5 |
| Damage | 1d6 |
| Power Source | Heat |
| Heat Capacity | 5 |
| Heat / Shot | 1 |
| Modifiers | **Elemental: Fire** |
| Spawn | D:5–D:12, chance 0.12, price 180g |

Today: purely cosmetic (orange projectile + "(Fire)" log tag). Once
resistances exist, this becomes a meaningful pick.

### 1.7 Monk's Crook

The "Melee" modifier flex. A ranged weapon that doubles as a respectable
melee stick, rewarding a player who runs in close.

| Field | Value |
| --- | --- |
| Glyph / Color | `}` / ash-wood |
| Slot | MainHand, two-handed |
| Range / Increment | 5 / 4 |
| Damage (ranged) | 1d6 |
| Damage (melee bump) | 1d8, Medium weight, STR-based |
| Power Source | Heat |
| Heat Capacity | 5 |
| Heat / Shot | 1 |
| Modifiers | **Improved Melee** |
| Spawn | D:3–D:10, chance 0.1, price 140g |

Normal firearms bump-attack for 1d4 pistol-whip damage. The Crook's
`melee_profile` overrides this with a real quarterstaff profile — still
bypasses Aegis (it's a bump), so it's the melee-ranged switch weapon of
choice against aegis-having mobs.

---

## 2. Ammunition

### 2.1 Heavy Ammo

Stackable fungible item for Heavy-class weapons.

| Field | Value |
| --- | --- |
| Glyph / Color | `"` / brass |
| Slot | (not equipped — consumed directly) |
| Stack | 1–12 per pickup |
| Modifiers | Consumable |
| Spawn | D:3–D:15, chance 0.35, price 3g each |

Spawns in caches of 4–8 near Heavy Rifle drops so the weapon is usable at
pickup.

---

## 3. Aegis-Having Enemies

A handful of enemies need Aegis to make Tachyonic/Scatter/Shredding-break
loops meaningful. These are new entries for `content.json`.

### 3.1 Rampart Sentry

Mid-tier enforcer. Moderate HP, moderate AV, solid Aegis. The target you
want the Tachyon Lance for.

| Field | Value |
| --- | --- |
| Glyph / Color | `S` / steel blue |
| HP / Defense / Power | 22 / 3 / 6 |
| **Aegis** | 6 |
| Faction | Orcs (mechanized garrison flavor) |
| Personality | Tactical |
| Viewshed | 10 |
| Ranged (innate) | 8 |
| Spawn | D:4–D:12, chance 0.2, xp 60 |

Behavior notes: Tactical + Ranged → prefers to kite. With the new
`PartialCover` feature (§2 of implementation-plan), seeding these in rooms
with debris lets them turtle behind cover while firing.

### 3.2 Vault Drone

Low HP, high Aegis. A glass shield — breaks fast if you can push through
the force field.

| Field | Value |
| --- | --- |
| Glyph / Color | `d` / cyan |
| HP / Defense / Power | 8 / 1 / 4 |
| **Aegis** | 10 |
| Faction | Animals (Vaults custodian flavor) |
| Personality | Brave |
| Viewshed | 8 |
| Spawn | D:5–D:15 (Vaults branch), chance 0.25, xp 40 |

Tachyonic specifically shines here: a 5-raw Lance hit breaks the full
10-Aegis in one shot.

### 3.3 Temple Bastion

Boss-adjacent. Carried into D:6+ as a miniboss-grade encounter or paired
with Temple Priests.

| Field | Value |
| --- | --- |
| Glyph / Color | `B` / gold |
| HP / Defense / Power | 60 / 5 / 10 |
| **Aegis** | 12 |
| Faction | Temple |
| Personality | Brave |
| Viewshed | 10 |
| Spawn | D:6–D:12, chance 0.08, xp 250 |

Shielded *and* armored. This is where Shredding earns its keep: drop the
Aegis first (Tachyonic), then start stacking Shredded to punch through
the AV.

---

## 4. Partial Cover Placement

Not a content entry per se, but worth spelling out for tuning:

- Seed 1–3 `PartialCover` entities per room on average, biased toward rooms
  with enemies whose personality is `Tactical`.
- Never block stairs, merchants, or choke points — cover should create
  interesting fights, not stall the game.
- In the Vaults branch, cover is rarer but paired with Rampart Sentries
  for a pronounced cover-shooter feel.

---

## 5. Playtest Scenarios

Scenarios to run once implementation lands. Each tests a specific
interaction and can be seeded deterministically via the debug console.

### 5.1 "The Bastion Dance"

- Location: D:6 temple room
- Enemy: 1× Temple Bastion
- Player loadout: Tachyon Lance + Heavy Rifle (Shredding)
- Expected flow:
  1. Opening Lance shots crack Aegis in 2–3 hits.
  2. Switch to Heavy Rifle; stack Shredded to 3–5.
  3. Close for Monk's Crook bump attacks (bypass Aegis on any residual
     re-shielding).

Validates: Tachyonic against Aegis, Shredding under decay, weapon switch
ergonomics.

### 5.2 "Cover Fire"

- Location: D:4 orc garrison, seeded with Partial Cover
- Enemies: 3× Rampart Sentry, all behind cover
- Player loadout: Scattergun + Phoenix Repeater
- Expected flow:
  1. Repeater pot-shots at range — +2 DC from cover makes hit rate painful.
  2. Close to clear cover-to-cover; Scattergun step-down damage still
     lethal inside 2 increments.

Validates: Partial Cover DC math, Scatter step-down ladder.

### 5.3 "Heat Budget"

- Location: D:3 extended corridor, seeded with 6× Goblin
- Player loadout: Carbine only (no melee fallback)
- Expected flow:
  1. Three bursts = one full heat cycle, ~2 goblins down.
  2. Must reposition and skip a turn while venting.
  3. Recover and finish.

Validates: Burst + Heat + vent lockout + passive cooldown pacing.

### 5.4 "Aegis Drought"

- Player has aegis 5/5. Step into a trap (10 damage).
- Expected: Aegis unchanged (trap is `Contact` route per §1.2), HP loses
  8 (AV 2), *no* `AegisDrought` applied.
- Then take a projectile hit for 3 raw.
- Expected: Aegis drops to 2, `AegisDrought(5)` applied, no HP loss.

Validates: Trap routing, drought trigger semantics, force-field framing.
