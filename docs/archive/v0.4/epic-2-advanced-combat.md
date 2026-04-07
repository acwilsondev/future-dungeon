# Epic 2: Advanced Combat Resolution (DC System)

This epic overhauls combat resolution to use a standard Difficulty Class (DC) system.

## User Stories

### Melee Attack Rolls

- **As a player,** I want to make a `1d20 + STR mod` roll to hit enemies with heavy weapons, or `1d20 + DEX mod` for light weapons.
- **As a player,** I want my attack roll to succeed if it meets or exceeds the target's Dodge DC (`10 + DEX mod`).

### Critical Hits and Misses

- **As a player,** I want a roll of 1 to always be a miss.
- **As a player,** I want a roll of 20 to always be a hit and deal double damage.

### Damage Calculation

- **As a player,** I want my damage to be calculated as `weapon_roll + attribute_mod - target_AV`.
- **As a player,** I want to always deal at least 1 damage on a hit.

### Off-Hand Attacks

- **As a player,** I want a chance to make a secondary attack with my off-hand weapon, based on my attribute modifiers.

## Developer Goals

- Implement `CombatResolution` as a dedicated system to handle attack rolls, criticals, and damage reduction.
- Add support for weapon `weight` (light, medium, heavy) to determine the relevant attribute for attacks.
- Update the bump action to trigger the new combat resolution flow.
