# Epic 6: The Great Quest (Victory Condition)

This epic provides a definitive "Win State" and a primary motivation for the player.

## User Stories

### The Ultimate Goal
- **As a player,** I want to find the "Amulet of the Ancients" on the final floor of the dungeon.
- **As a player,** I want to have to fight my way back *up* to the first floor and exit the dungeon to win the game.

### Victory & Defeat Screens
- **As a player,** I want a satisfying victory screen that summarizes my journey, including monsters killed, gold collected, and my final level.

### End-game Pressure
- **As a player,** I want the dungeon to become more dangerous or "unstable" once I have the Amulet, creating a tense race to the exit.

## Developer Goals
- Implement a global `Victory` state and an `Escaping` state that modifies spawning rules.
- Add a summary screen that parses the session log and stats.
