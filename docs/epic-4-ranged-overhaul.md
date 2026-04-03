# Epic 4: Ranged Combat & Ammunition

This epic overhauls ranged combat to include dedicated targeting, range increments, and ammunition consumption.

## User Stories

### Targeted Ranged Attacks

- **As a player,** I want to activate my ranged weapon by pressing `f`.
- **As a player,** I want to select a target using a visual targeting cursor.

### Range-Based Accuracy

- **As a player,** I want to have a standard attack roll at short range, with penalties (disadvantage) for each full increment of range beyond the weapon's base range.

### Ammunition Consumption

- **As a player,** I want to consume ammunition (arrows, bolts) from my Ammo slot when making ranged attacks.
- **As a player,** I want my ranged weapon to be unusable if I have no matching ammunition equipped.

## Developer Goals

- Create a `TargetingSystem` to handle the 'f' key flow and cursor movement.
- Implement the range increment logic (rolling 1d20 twice and taking the lower result for each increment).
- Ensure ranged attacks correctly consume items from the `Ammo` slot.
