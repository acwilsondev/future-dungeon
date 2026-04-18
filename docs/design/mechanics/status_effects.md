# Status Effects

Here we define status effects that may be applied to an object.

## Status Effect Design Registry

**Parametric Types**: If a type listed below has `<type parameters>`, it is assumed these are realized in content. Our content system only supports two int parameters.

For example `Fortified<Stat>(M, D)` must actually be implemented as `FortifiedStrength(M, D)`, etc.

| Status Effect           | Type         | Description                                                                                                                |
| :---------------------- | :----------- | :------------------------------------------------------------------------------------------------------------------------- |
| `Aegis(M, D)`           | Buff         | Grants **M** temporary health for **D** turns.                                                                             |
| `Drought<Pool>(D)`      | Debuff       | Cannot recover **Pool** for **D** turns.                                                                                   |
| `Anchored(D)`           | Debuff       | Target cannot be moved or teleported for **D** turns. (Including by self). STR save each turn to lose the status.          |
| `Blinded(M, D)`         | Debuff       | Reduces vision radius to **M** (usually 0) for **D** turns. WIS save each turn to lose the status.                         |
| `Confusion(M, D)`       | Debuff       | **M%** chance to move/attack randomly for **D** turns. WIS save each turn and when damaged to lose the status.             |
| `Crushing(M, D)`        | Debuff       | Increases movement/action cost by **M** for **D** turns.                                                                   |
| `Damage<Type>(M, D)`    | DoT          | Deals **M** damage of **Type** per turn for **D** turns.                                                                   |
| `Drained<Stat>(M, D)`   | Debuff       | Reduces specific **Stat** by **M** for **D** turns.                                                                        |
| `Flicker(M, D)`         | Buff         | **M%** chance to ignore any incoming effect for **D** turns.                                                               |
| `Fortified<Stat>(M, D)` | Buff         | Increases specific **Stat** by **M** for **D** turns.                                                                      |
| `Light(M, D)`           | Buff/Utility | Illuminates a radius of **M** for **D** turns.                                                                             |
| `Mired(M, D)`           | Hazard       | Exiting the current tile costs **M** additional turns for **D** turns.                                                     |
| `Phasing(D)`            | Buff         | Ignore wall collisions for **D** turns.                                                                                    |
| `Poisoned(D)`           | Debuff       | Roll attacks with disadvantage for **D** turns.                                                                            |
| `Refraction(M, D)`      | Buff         | When hit by a projectile, **M**% chance to redirect it to the nearest enemy for **D** turns.                               |
| `Regen(M, D)`           | Healing      | Restores **M**/10 health per turn for **D** turns.                                                                         |
| `Stunned(M, D)`         | Debuff       | Cannot take actions for **D** turns (**M** as recovery threshold). WIS save each turn and when damaged to lose the status. |
| `Warped(M, D)`          | Debuff       | Randomly teleports target **M** tiles every turn for **D** turns.                                                          |

---

## Technical Notes for Implementation

### 1. The Magnitude (M) Variable

In your system, **Magnitude** should be context-aware based on the Hook:

* **Percentage:** For `Fortified` or `Flicker`.
* **Flat Value:** For `Shielded` or `Regen`.
* **Radius:** For `Light` or `Infected` propagation.

### 2. Missing "Hardened" Definition

Based on the **Iron** table, you need a definition for **Hardened**. Given the "Grit/Industrial" vibe, I suggest:
> **Hardened(M, D):** Reduces all incoming non-magical damage by a flat **M** value for **D** turns.

### 3. The "Refraction" Effect

This was marked as "Unsure" in your Solari notes. To fit the **Additive/Vision** theme of Orange:
> **Refraction(M, D):** When hit by a beam or projectile, **M%** chance to redirect it toward the nearest enemy for **D** turns.

### 4. Duration (Y) Logic

* **D = 0:** Effect expires at the end of the current turn.
