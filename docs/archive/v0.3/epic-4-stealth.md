# Epic 4: Stealth & Sound Mechanics

This epic introduces a new dimension of gameplay centered around noise and detection.

## User Stories

### Noise Levels
- **As a player,** I want different actions to create different amounts of noise (e.g., walking is quiet, fighting is loud, breaking a door is very loud).
- **As a player,** I want to see a "Noise Indicator" that tells me how likely I am to be heard by nearby monsters.

### Alert States
- **As a player,** I want to be able to "sneak" past sleeping monsters or use noise (like throwing a stone) to distract them and lure them away from their posts.

### Tactical Stealth
- **As a player,** I want to deal extra damage when attacking a monster that hasn't detected me yet (Sneak Attack).

## Developer Goals
- Implement a "Sound Propagation" algorithm that radiates from a source and is muffled by walls.
- Add an `AlertState` to the AI (Sleeping, Curious, Aggressive).
