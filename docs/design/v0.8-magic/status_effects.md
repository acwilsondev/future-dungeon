# Status Effects

Here we define status effects that may be applied to an object.

This is how live status effects are represented in the ECS:

```yaml
StatusEffect:
    owner: entity id                    # the entity that created the effect
    affected: entity id                 # the entity that is subject to the effect
    type: string representing the type  # a descriptive string meant to identify the status effect kind
    duration: optional int              # a duration of the effect in turns
    magnitude:                          # magnitude is contextual, depending on the effect type
        dice: int                       # how many die to roll
        sides: int                      # size of die to roll
        bonus: int                      # static modifier to add to the roll
    recovery_save: optional { STR | DEX | CON | INT | WIS | CHA }  # rolled each turn; success removes the status early
    ...                                 # additional fields as needed
```

In content files, we use a smaller specification of the effect.

```yaml
Armored:
    duration: optional int              # a duration of the effect in turns
    magnitude: !dice "1d6+2"            # valid die roll string representing the magnitude
        # Full form:  /^(?<dice>\d+)d(?<sides>\d+)(?:[+-](?<bonus>\d+))?$/   e.g. "2d6+3"
        # Flat form:  /^(?<flat>\d+)$/                                        e.g. "50"
        # A plain integer is treated as a flat constant (no dice rolled).
    recovery_save: optional { STR | DEX | CON | INT | WIS | CHA }
```

Status effects are processed at the beginning of a creature's turn.

## Status Effect Flow

1. `Affected` entity turn begins.
2. Status effect logic is applied one time.
3. `Duration` is decremented.
4. If `Duration` is 0, the Status Effect is removed.
5. Entity turn proceeds as normal.

## Status Effect Design Registry

**Parametric Types**: If a type listed below has `<type parameters>`, it is assumed these are realized in content. Our content system only supports two int parameters.

For example `Fortified<Stat>` must actually be implemented as `FortifiedStrength`, etc.

**Recovery Saves**: Where the description says "X save each turn to lose the status", this corresponds to setting `recovery_save: X` in the content file for that status. See `effects.md` for the distinction between application saves and recovery saves.

| Status Effect     | Type         | Description                                                                                                                |
| :---------------- | :----------- | :------------------------------------------------------------------------------------------------------------------------- |
| `Armored`         | Buff         | Increases armor by **M** for **D** turns.                                                                                  |
| `ManaDrought`     | Debuff       | Cannot recover any Mana (all colors) for **D** turns.                                                                      |
| `Anchored`        | Debuff       | Target cannot be moved or teleported for **D** turns. (Including by self). STR save each turn to lose the status.          |
| `Blinded`         | Debuff       | Reduces vision radius to **M** (usually 0) for **D** turns. WIS save each turn to lose the status.                         |
| `Confusion`       | Debuff       | **M%** chance to move/attack randomly for **D** turns. WIS save each turn and when damaged to lose the status.             |
| `Crushing`        | Debuff       | Increases movement/action cost by **M** for **D** turns. STR save each turn to lose the status.                            |
| `Damage<Type>`    | DoT          | Deals **M** damage of **Type** per turn for **D** turns.                                                                   |
| `Drained<Stat>`   | Debuff       | Reduces specific **Stat** by **M** for **D** turns.                                                                        |
| `Flicker`         | Buff         | **M%** chance to ignore any incoming effect for **D** turns.                                                               |
| `Fortified<Stat>` | Buff         | Increases specific **Stat** by **M** for **D** turns.                                                                      |
| `Gravity`         | Buff/Debuff  | Pull all objects in radius **M** towards the affected for **D** turns.                                                     |
| `Light`           | Buff/Utility | Illuminates a radius of **M** around affected for **D** turns.                                                             |
| `LoseLife`        | DoT          | Typeless loss of Health **M** for **D** turns.                                                                             |
| `Mired`           | Hazard       | Exiting the current tile costs **M** additional turns for **D** turns.                                                     |
| `Phasing`         | Buff         | Ignore wall collisions for **D** turns.                                                                                    |
| `Poisoned`        | Debuff       | Roll attacks with disadvantage for **D** turns.                                                                            |
| `Refraction`      | Buff         | When hit by a projectile, **M**% chance to redirect it to the nearest enemy for **D** turns.                               |
| `Regen`           | Healing      | Restores **M** health per turn for **D** turns.                                                                            |
| `Stunned`         | Debuff       | Cannot take actions for **D** turns (**M** as recovery threshold). WIS save each turn and when damaged to lose the status. |
| `Warped`          | Debuff       | Randomly teleports target **M** tiles every turn for **D** turns. CON save each turn to lose the status.                   |
| `LifeDrain`       | DoT          | Deals **M** necrotic damage per turn for **D** turns, healing the owner by the final damage amount.                        |
