# Color Pie

## The Pentagonal Spectrum

### 1. Orange: The Solari (Holy / Radiant / Fusion)

- **The Vibe:** The blinding light of a newborn star. High-tech paladins and solar monks.
- **Gameplay:** Buff-heavy. Focuses on **Shielding**, **Damage Multipliers**, and **Radiance** (illuminating the map and burning enemies who look directly at you).
- **References:** Jedi, Protoss
- **Preferred Damage Types:**
  - Fire

| **Effect Name** | **Engine Behavior**              | **Save**             | **Status** |
| --------------- | -------------------------------- | -------------------- | ---------- |
| **Radiance**    | Target(s) gain Light(X, Y).      | None                 | Approved   |
| **Fortify**     | Target(s) gain Fortified(X, Y)   | None                 | Approved   |
| **Aegis**       | Targets(s) gain AegisBoost(X, Y) | None                 | Approved   |
| **Refraction**  | Target(s) gain Refraction(X, Y)  | None                 | Approved   |
| **Burn**        | Target(s) gain Burning(X, Y)     | DEX                  | Approved   |
| **Fire Damage** | Deals X damage.                  | DEX for half damage. | Approved   |

### 2. Purple: The Nihil (Sith-like / Shadow / Gravity)

- **The Vibe:** The crushing weight of a black hole. Forbidden knowledge and ego-dissolution. Warlocks, mystic assassins.
- **Gameplay:** Debuff-heavy. Focuses on **LifeDrain**, **Slows**, **Confusion**, and **Entropy** (armor degradation). It’s about making the enemy too weak to fight back.
- **References**: Sith, Dark Protoss
- **Preferred Damage Types:**
  - Life Drain

| **Effect Name**         | **Engine Behavior**                                                          | **Save**           | **Status**               |
| ----------------------- | ---------------------------------------------------------------------------- | ------------------ | ------------------------ |
| **Drain** [STR, DEX...] | Adds a DrainedAbility(ABILITY, X, Y) status to one target.                   | Corresponding Stat | Hold for balancing.      |
| **Gravity Well**        | Creates a Gravity Well at target location.                                   | None               | Approved                 |
| **Entropy**             | Permanent reduction in target `Armor_Value` or `Weapon_Die`. Non-restorable. | CON                | Hold for system/balance. |
| **Gloom**               | Add a Blinded(X, Y) status to targets.                                       | WIS                | Approved                 |
| **Confusion**           | Add a Confusion(X, Y) status to targets.                                     | WIS                | Approved                 |
| **Crush**               | Target(s) gain Crushing(X, Y)                                                | CON                | Approved                 |
| **Crush Damage**        | Deals X damage.                                                              | CON for half       | Approved                 |
| **Lose Life**           | Typeless reduction in Health that bypasses AV.                               | CON for half       | Approved                 |

### 3. Iron: The Kinetic (Mundane / Industrial / Grit)

- **The Vibe:** Cold steel, gunpowder, hydraulics, and manual labor. The "Human" element in a cosmic world.
- **Gameplay:** The baseline. Focuses on **Reliability**, **Physical Defense**, and **Tactical Tools** (grappling hooks, barricades, smoke grenades). It doesn't rely on magic so it's immune.
- **References:** Terrans
- **Wider Effect Access:** Generally, Iron color has access to whatever effects can be explained by technology, with proportionately higher costs.
- **Preferred Damage Types:**
  - Physical
   Fire

| **Effect Name**         | **Engine Behavior**                                                                                                                                                  | **Save**   | **Status**           |
| ----------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- | -------------------- |
| **Impact**              | Moves entity `X` tiles away. If targets hit a wall, they take `1d6` damage per tile they could not move. If the target is a point, all entities within X are pushed. | CON        | Approved             |
| **Barricade**           | Creates a wall for Y turns.                                                                                                                                          | None       | Needs targeting rule |
| **Bleed**               | Grants Damage<Physical>(X, Y).                                                                                                                                       | CON or DEX | Approved             |
| **Create [Type] Cloud** | Creates a Cloud of a given type that disperses.                                                                                                                      | None       | Needs Clouds         |
| **Hardened**            | Unknown                                                                                                                                                              |            | Needs definition.    |

### 4. Cyan: The Aetheric (Alien / Weird Nature / Fae)

- **The Vibe:** Bioluminescent jungles, crystalline growths, and non-Euclidean geometry.
- **Gameplay:** The "Trickster" color. Focuses on **Teleportation**, **Cloning**, **Phasing** (walking through walls), and **Probability**. It’s less about damage and more about breaking the "rules" of the grid.
- **Preferred Damage Types:**
  - Psychic
  - Force
  - Physical (Slashing, Piercing)

| **Effect Name** | **Engine Behavior**                                                                 | **Save**             | **Approved** |
| --------------- | ----------------------------------------------------------------------------------- | -------------------- | ------------ |
| **Phase**       | Gains Phasing(X)                                                                    | None                 | Approved     |
| **Blink**       | Instantaneous change of `Entity` coordinates. Bypasses all intervening traps/tiles. | CHA if targets enemy | Approved     |
| **Clone**       | Creates a `Decoy_Entity` with 1 HP that shares the creator's ASCII symbol.          | None                 | Approved     |
| **Flicker**     | Gains Flicker(X, Y) (ignore attacks, effects)                                       | None                 | Approved     |
| **Anchor**      | Gains Anchored(X)                                                                   | STR                  | Approved     |
| **Warp**        | Gains Warped(X, Y)                                                                  | CON                  | Approved     |

### 5. Emerald: The Bio-Mass (Evolution / Toxic / Hive)

- **The Vibe:** Corrosive fluids, chitinous plating, and rapid mutation.
- **Gameplay:** Sustained pressure. Focuses on **Damage-over-Time (Poison/Acid)**, **Summoning** (spawn mini-drones or bio-larvae), and **Self-Mutation** (growing wings or extra limbs for a few turns).
- **References:** Zerg
- **Favored Damage Types:**
- Physical
- Poison
- Acid

| **Effect Name** | **Engine Behavior**                                                                 | **Save** | **Approved** |
| --------------- | ----------------------------------------------------------------------------------- | -------- | ------------ |
| **Spawn**       | Creates `Minion_Entities` that follow basic `Seek_Player` or `Protect_Caster` AI.   |          |              |
| **Mire**        | Increases `Turn_Cost` to exit a tile (e.g., moving out of this tile takes 2 turns). |          |              |
| **Corrode**     | Damage that scales over time (`Damage = Base + (Turns_Active * Scale)`).            |          |              |
| **Mutation**    | Grants a random `Iron`, `Cyan`, or `Orange` effect for a limited `Turn_Duration`.   |          |              |
| **Poison**      | Grants Poisoned(X, Y)                                                               |          |              |
| **Heal**        | Recover X health for Y turns.                                                       |          |              |

## 1. The Foundation: Iron, Steel, & Dirt (Baseline)

_The "Mundane" Layer. If it exists in a standard roguelike without magic, it lives here._

- **Availability:** Constant. Found in 90% of tiles, items, and common mobs.
- **Systemic Nature:** **Hard Collisions & Physics.**
  - **Abilities (Tool-Based):** These aren't "spells" cast from the soul; they are `Item_Actions`.
  - **Effects:** _Bleeding, Stunning, Breaking, Blocking, Weight._
  - **The "Electrical" Exception:** Logic-based effects (EMP, Circuit-Shorting) that target other Iron-tier tech.
- **Player Interface:** Reliability. Iron doesn't fail, doesn't require "Energy," and is the only bucket that provides **Passive Armor Class**.

---

## 2. The Core Conflict: Solari & Nihil (High Tier)

_The "Metaphysical" Layer. These colors drive the narrative and late-game complexity._

- **Availability:** Rare/Factional. Items are usually "Artifact" quality. Spells require specific alignment or high-tier progression.
- **Systemic Nature:** **State Modification & Resource War.**
  - **Solari (Radiant/Fusion):** Focuses on **Additive States**. Creating light, adding shields, increasing magnitude. It is the "Overclock" of the world.
  - **Nihil (Shadow/Gravity):** Focuses on **Subtractive States**. Removing light, draining magicka, reducing armor. It is the "Underclock" of the world.
- **Player Interface:** High Investment. Choosing a side (or trying to balance both) dictates the late-game "verbs" available to the player.

---

## 3. The Tertiary: Aetheric & Biomass (The "Wild Cards")

_The "Environmental" Layer. These function as the "Flavor" or "Hazards" that disrupt the Core Conflict._

- **Availability:** Uncommon/Zonal. Found in specific biomes (the "Bioluminescent Jungle" or the "Non-Euclidean Rift").
- **Systemic Nature:** **Grid Violation & Self-Propagation.**
  - **Aetheric (Fae/Weird):** Logic that breaks the ASCII grid. Teleportation, swapping, and phasing. It's the "Engine Glitch" bucket.
  - **Biomass (Hive/Toxic):** Logic that saturates the ASCII grid. Spreading pools of acid, spawning low-AI fodder, and evolving. It's the "Cellular Automata" bucket.
- **Player Interface:** Reactive. Players rarely "build" for these, but they must have a plan to _survive_ them. A rare Aetheric item might be a "get out of jail free" card, but it's not a reliable primary strategy.

## Levels by Faction/Epicness

This scaling creates a clear hierarchy of "threat density." As the value increases, the environments shift from grounded, mechanical corridors to reality-warping, biological god-realms.

### The Epicness Hierarchy

| Value | Tier | Description |
| :--- | :--- | :--- |
| **1** | **Mundane** | Low-stakes, industrial, predictable physics. |
| **2-3** | **Specialist** | Mixing standard tech with high-energy or biological hazards. |
| **4** | **Extreme** | Colliding celestial powers or high-tech cosmic forges. |
| **6** | **Reality-Warping** | Where the laws of physics and biology begin to dissolve. |
| **9** | **God-Lair** | The apex of the spectrum; non-Euclidean, living nightmares. |

---

### Tier 1: The Mundane (1 pt)

- **Iron / Iron (1):** _The Grease Pit, Low-Sector Sump, Bolt-Cutter Bay._

### Tier 2-3: The Hybrid Zones (2-3 pts)

- **Iron / Orange (2):** _The Blast-Furnace Nave, Smelting Altar, Steam-Paladin Barricade._
- **Iron / Purple (2):** _Lead-Lined Sinkhole, The Gravity Press, Cold-Iron Oubliette._

- **Iron / Cyan (3):** _The Glitch-Steel Gantry, Phase-Wire Duct, Logic-Ghost Lab._
- **Iron / Emerald (3):** _The Sludge-Pump Hive, Rust-Blight Pit, Hydraulic Maw._

### Tier 4-6: The Transcendent Halls (4-6 pts)

- **Orange / Orange (4):** _The Solar Core-Chamber, Throne of the Unblinking Eye, Nova Altar._
- **Purple / Purple (4):** _The Singularity Vault, Event Horizon Crypt, Nihil-Point Zero._
- **Orange / Purple (4):** _The Penumbra Spire, Twilight Fracture, Hall of the Dying Star._

- **Orange / Cyan (6):** _The Aurora Glass-Walk, Prismatic Infinite, Chronos-Light Atrium._
- **Orange / Emerald (6):** _The Photosynth Cathedral, Amber Spore-Hold, Solar-Infect Garden._
- **Purple / Cyan (6):** _The Void-Crystal Reef, Entropy-Flicker Corridor, Echoes of the End._
- **Purple / Emerald (6):** _The Festering Black-Well, Necrotic Hive-Mind, Entropy-Mire._

### Tier 9: The God-Lairs (9 pts)

- **Cyan / Cyan (9):** _The Fractal Singularity, Non-Euclidean Engine, The Pale Dream-Lattice._
- **Emerald / Emerald (9):** _The Great Progenitor-Sac, Heart of the World-Eater, The Ever-Mutating Core._
- **Cyan / Emerald (9):** _The Biolume Infinite, Reality-Warping Hatchery, The Phasing God-Brain._

### The Systemic Interaction Matrix

| | **Iron** (Physics) | **Orange** (Additive) | **Purple** (Subtractive) | **Cyan** (Grid Logic) | **Emerald** (Propagation) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Iron** | **Hard Surface**: Predictable physics and manual labor. | **Augmentation**: High-tech paladins; machines powered by solar "Fortify" logic. | **Heavy Industry**: "Entropy" applied to steel; armor degradation and crushing weight. | **Technical Glitch**: "Phase-wire" and "Logic-ghosts" that bypass physical barriers. | **Bio-Mechanical**: "Rust-blight" and hydraulic maws; living machines. |
| **Orange** | — | **Pure Radiance**: Absolute vision and maximum shielding. | **The Penumbra**: A war of additive vs. subtractive states; flickering light. | **Prismatic**: Light that bends space; "Blink" effects tied to "Illumination". | **Golden Charnel**: Titanic overgrowth fueled by "Radiance"; the "False Security" garden. |
| **Purple** | — | — | **The Void**: Total resource siphon and absolute "Gloom". | **Dark Matter**: Gravity traps that "Anchor" entities while draining their life. | **Necrotic**: "Infect" logic paired with "Entropy"; a spreading, rotting decay. |
| **Cyan** | — | — | — | **Non-Euclidean**: Pure "Flicker" and "Phase" logic; the grid dissolves. | **The Weird**: "Biolume Infinite"; phasing monsters that "Spawn" from the walls. |
| **Emerald** | — | — | — | — | **The Great Hive**: Infinite "Mutation" and self-scaling "Corrode" logic. |

---

### Key Product Gists

- **Iron Products (The Hardware):** Focus on **Positioning** and **Reliability**. These interactions usually result in physical obstacles like `Barricades` or `Impact` effects that ignore magic.
- **Orange/Purple Products (The State War):** Focus on **Resource War** and **Vision**. These products manipulate the `FOV Engine` and `Resource Siphon` hooks to control the player's capacity to act.
- **Cyan/Emerald Products (The Rule Breakers):** Focus on **Grid Violation** and **Scaling**. These products use `Collision Bypass` and `Propagation` to ensure the player is never truly safe, even behind "walkable: false" tiles.
- **The Orange/Emerald "False Security":** This specific product utilizes `Radiance` (revealing the map) to lure the player into a sense of clarity, only to trigger `Mutation` and `Infect` hooks from "Titans" hiding in plain sight.
