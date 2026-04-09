# Epic 3: Debug Console

This epic adds a powerful in-game tool for developers to manipulate the game state, facilitating faster testing and debugging of complex mechanics.

## User Stories

### Real-time Interaction

- **As a developer,** I want to open a console overlay by pressing a dedicated key (e.g., `~`).
- **As a developer,** I want to type commands into the console to affect the game world immediately.

### Essential Commands

- **`spawn [item_name]`:** Spawn a specific item at the player's feet.
- **`teleport [level]`:** Instantly move the player to a specific dungeon level.
- **`heal`:** Fully restore the player's HP and remove status effects.
- **`reveal`:** Reveal the entire map for the current floor.
- **`levelup`:** Give the player enough XP to level up.
- **`god`:** Toggle invincibility for testing combat without risk of death.

## Technical Tasks

- Implement a `DebugConsole` UI component using `ratatui`.
- Create a `CommandParser` to handle text input and map it to internal functions.
- Add a `Debug` game state to handle console input and rendering.
- Implement the backend logic for all requested debug commands.
