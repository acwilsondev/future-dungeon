# Epic 1: Multi-level Dungeon Depth

This epic focuses on expanding the game world from a single level to an infinite (or very deep) descent.

## User Stories

### Level Transitions
- **As a player,** I want to find stairs leading down to deeper levels, so that I can progress through the dungeon.
- **As a player,** I want to be able to go back up stairs to previous levels, and find them exactly as I left them (Persisted levels).

### Depth Scaling
- **As a player,** I want the monsters on deeper levels to be tougher and the loot to be more rewarding, so that the challenge matches my progression.
- **As a developer,** I want a system that scales monster stats and item quality based on the current `dungeon_level`.

### Level Generation
- **As a game designer,** I want the `MapBuilder` to trigger for every new floor reached, ensuring a fresh layout for every level.
