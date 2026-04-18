# Progression

**Status:** Development Complete

To make the **1–20 Level Arc** work across **99 Floors**, we need to calculate a "Monster Value" per floor that feeds into a quadratic XP curve.

If we assume the player clears **~80%** of a floor, we can bake that "missing 20%" into the curve so they don't feel forced to full-clear every single corner just to stay on pace.

## 1. The XP Curve Formula

To achieve the "Fast Start, Long Tail" feel, we’ll use a standard exponential growth formula for the **Total XP Required** for each level.

$$XP_{\text{Required}} = \text{Base} \times (\text{Level}^{\text{Exponent}})$$

* **Base:** 100
* **Exponent:** 2.5 (This creates the "5e-style" slowdown in the mid-game).

| Level | Total XP Required | Delta (XP to gain) | Floor Target (approx) |
| :--- | :--- | :--- | :--- |
| **1** | 0 | 0 | Floor 1 |
| **2** | 100 | 100 | Floor 2 |
| **3** | 300 | 200 | Floor 5 |
| **5** | 1,000 | ~400 | Floor 10 |
| **10** | 5,500 | ~1,200 | Floor 35 |
| **15** | 15,000 | ~2,500 | Floor 70 |
| **20** | 32,000 | ~4,500 | Floor 95+ |

---

## 2. Floor "Budget" (Monster Density)

Each floor has a **Total XP Value** ($V_f$) based on the monsters spawned.

* **The 80% Rule:** We calculate the level-up beats assuming the player only harvests **0.8** of the available $V_f$.
* **The "Monster Value" Scaling:** As the player goes deeper, monsters aren't just tougher; they are worth more.

**Formula for Floor Value ($V_f$):**
$$V_f = 100 + (f \times 20)$$
*(Where $f$ is the Floor Number)*

* **Floor 1:** 120 Total XP available. (Player gets ~96). **Result:** Hits Level 2 quickly.
* **Floor 50:** 1,100 Total XP available. (Player gets ~880). **Result:** Takes ~2.5 floors to gain a single level in the mid-game.
* **Floor 90:** 1,900 Total XP available. (Player gets ~1520). **Result:** Takes ~3 floors to hit those final "Ascension" levels.

---

## 3. Equipment "Drop Beats" (The Combinatorial Explosion)

Since the XP slows down, **Item Level (iLvl)** must pick up the slack. We can tie the "Quality" of items found to the Floor Number.

| Floor Range | Drop Tier | Combination Potential |
| :--- | :--- | :--- |
| **1 – 15** | **Tier 1 (Common)** | Basic AV (Chainmail, Wooden Shield). |
| **16 – 40** | **Tier 2 (Uncommon)** | First "Attribute" rings (+1 STR) and basic Proc items. |
| **41 – 70** | **Tier 3 (Rare)** | Synergistic gear (e.g., "Boots of the Skirmisher" + DEX-based shields). |
| **71 – 99** | **Tier 4 (Relic)** | High-break-chance glass cannons and "Artifact" torso pieces. |

---

## 4. The "Fighter-to-Wizard" Math Check

If a [Fighter](player.md) decides to pivot on Floor 40 (Level 10):

* They have **10 [Attribute Points](player.md)** to spend.
* They need to hit **INT 13** just to have a **+1 Mod** (to start learning basic scrolls).
* **The Risk:** Because the XP curve has flattened out by Floor 40, they will only get about **10 more points** for the rest of the game.
* **The Equipment Fix:** They MUST find "Combinatorial" gear (e.g., a +2 INT Amulet) to make the pivot viable, as the XP curve won't give them enough raw stat points to "catch up" naturally.

## 5. Summary for your Doc

> **XP & Scaling Logic**
>
> * **Efficiency Target:** Progression is balanced for **80% floor clearance**.
> * **XP Curve:** Exponential. Leveling is rapid until Floor 15 (Level 6), then slows significantly to allow **Equipment Combinations** to drive the mid-game.
> * **Monster Value:** Scales linearly per floor ($100 + 20 \times Floor$).
> * **Equipment Beats:** Items tiers unlock every ~25 floors, increasing the "Synergy Ceiling" while Attribute gains taper off.

Does this quadratic slowdown feel right for the "99-floor trek," or do you want the level-ups to stay more frequent throughout the mid-game?

Integrating a **Monster Level (ML)** and a **Monster Level Budget (MLB)** is the smartest way to handle procedural generation. It allows you to swap between "one big threat" and "a swarm of rats" while keeping the XP and difficulty of the floor mathematically consistent.

By using a floating-point ML (starting at **0.125**), you perfectly mirror the D&D "Challenge Rating" (CR) system, which is essential for making those early-game "Swarm" encounters feel balanced.

---

## 1. The Monster Level (ML) Scale

The ML represents the "Density of Threat."

| ML | Example Creature | XP Value ($ML \times 100$) |
| :--- | :--- | :--- |
| **0.125** | Giant Rat / Bat | 12.5 XP |
| **0.25** | Kobold Scavenger | 25 XP |
| **0.5** | Goblin Scout | 50 XP |
| **1.0** | Orc Warrior | 100 XP |
| **5.0** | Ogre Chieftain | 500 XP |
| **10.0** | Young Dragon | 1,000 XP |
| **30.0** | The Deep Lich (Boss) | 3,000 XP |

---

## 2. The Dungeon Level "Budget" (MLB)

Each floor ($f$) has a total "Points" pool to spend on spawning monsters. This budget ensures the player doesn't accidentally walk into ten Ogres on Floor 2.

**The Formula:**
$$MLB = 1 + (0.5 \times f)$$
*(Starting at 1.5 at Floor 1, scaling to ~50 at Floor 99)*

### How the Generator Spends the Budget

* **Floor 1 (MLB 1.5):** * *Option A:* 1x Orc Warrior (ML 1.0) + 4x Rats (ML 0.125).
  * *Option B:* 12x Rats (ML 0.125) — **The Swarm.**
* **Floor 50 (MLB 26.0):**
  * *Option A:* 5x Ogre Chieftains (ML 5.0) + 1x Orc (ML 1.0).
  * *Option B:* 1x Young Dragon (ML 10.0) + 32x Goblins (ML 0.5).

---

## 3. Floor XP vs. MLB

To sync this with our **80% Clear Rule**, the XP a monster grants must be tied directly to its ML.

* **Formula:** $XP = ML \times 100$.
* **Floor Yield:** If Floor 50 has a budget of 26.0 ML, the total XP available is **2,600**.
* **Player Harvest:** At 80% clear, the player gets **2,080 XP**.

---

## 4. Scaling the "To-Hit" and "AV" via ML

To keep the **Unbounded Attributes** from breaking the game, use the **ML** to set the baseline for monster stats:

* **Monster Dodge DC:** $10 + (ML \times 0.5) + \text{DEX}$.
* **Monster AV:** $ML \times 0.75$ (rounded).
* **Monster Damage:** $ML \times 2$ (base).

> **The Level 20 [Fighter](player.md) Check:**
> If a Level 20 [Fighter](player.md) has +12 [STR](player.md), they are rolling $1d20 + 12$.
> A **ML 20** monster has a Base Dodge DC of $10 + (20 \times 0.5) = 20$.
> The Fighter still needs to roll an **8 or higher** to hit, keeping the end-game tactical.

---

## 5. Summary for the Doc

### Monster Level (ML) & Budgeting

* **ML Range:** 0.125 (Swarm) to 30.0 (Boss).
* **XP Value:** $ML \times 100$.
* **MLB (Dungeon Level Budget):** $1 + (0.5 \times f)$.
* **Budget Logic:** The generator fills each floor by "purchasing" monsters until the MLB is exhausted. This allows for high-variance encounters (many weak vs. one strong) within the same XP target.
* **Stat Scaling:** Monster Dodge DC and AV scale linearly with ML to keep pace with player Attribute growth.

---

### One final thought for the "Fighter-to-Wizard" Pivot

Because the **MLB** grows linearly but the **XP Curve** is exponential, a character who spends their early points on [STR](player.md) but switches to [INT](player.md) mid-game will find themselves in a "Math Debt." They will be fighting **ML 15** monsters with **ML 5** magical potency.

Does this "Monster Level Budget" feel like it gives the procedural generator enough room to breathe?
