# Epic 1: Core Attributes & Modifier System

This epic replaces the simplified `CombatStats` with a robust, six-attribute system inspired by classic d20-based RPGs.

## User Stories

### Six Primary Attributes

- **As a player,** I want to have STR, DEX, CON, INT, WIS, and CHA attributes that define my character's capabilities.
- **As a player,** I want to see my attribute scores on the UI.

### Modifier Calculation

- **As a player,** I want my attribute scores to provide bonuses (modifiers) based on the formula `floor((Score - 10) / 2)`.
- **As a player,** I want these modifiers to be applied to all relevant checks (to-hit, damage, saves).

### XP and Attribute Progression

- **As a player,** I want to choose one attribute to increase by +1 whenever I level up.
- **As a player,** I want my HP to increase by `8 + CON mod` per level, with retroactively applied CON bonuses.

## Developer Goals

- Create an `Attributes` component to store the six primary scores.
- Implement a `get_modifier(score: i32) -> i32` helper function.
- Replace `CombatStats` in systems where `Attributes` is now the source of truth.
- Update the level-up system to allow attribute increases.
