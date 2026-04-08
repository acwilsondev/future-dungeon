# Epic 1: Dungeon Milestones (Rhythm)

This epic implements the "Dungeon Rhythm" logic, introducing special floor layouts and encounters at fixed intervals to structure the player's journey.

## User Stories

### Merchant & Altar Havens (Floors ending in 5)

- **As a player,** I want to encounter a safe single-room floor every 10 levels (5, 15, 25...) that contains a guaranteed Merchant and a Holy Altar.
- **As a player,** I want to use the Holy Altar to fully restore my HP.
- **As a player,** I want the 5th level Merchant to have twice as many items as the random merchant.

### Random Merchant Changes

- Merchants should spawn less frequently in normal dungeon levels (20% chance).
- Merchants that spawn in normal dungeon levels carry three items chosen at random for that floor.

### Boss Chambers (Floors divisible by 10)

- **As a player,** I want to face a powerful Boss monster every 10 levels (10, 20...) in a floor designed for a challenging encounter.
- **As a developer,** I want the level generator to support a `BossChamber` layout that focuses on a single large arena.

### Reset Shrines (Floors divisible by 20)

- **As a player,** I want to find a hidden Reset Shrine every 20 levels that allows me to re-spec my character while keeping my XP and items.

## Technical Tasks

- Implement a `DungeonRhythm` system that determines the floor type based on `dungeon_level`.
- Create a `SingleRoom` generator for Merchant Havens.
- Implement the `HolyAltar` component and interaction logic.
- Add logic to the level transition system to trigger the correct generator for milestones.