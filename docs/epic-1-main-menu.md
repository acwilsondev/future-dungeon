# Epic 1: The Great Gateway (Main Menu)

This epic implements the game's initial entry point based on the design in `docs/design/main_menu.md`.

## User Stories

### Interactive Main Menu

- **As a player,** I want to see a main menu when I start the game so I can choose between starting a new game, loading an existing one, or exiting.
- **As a player,** I want to see the game title "FUTURE DUNGEON" in high-contrast ASCII art.
- **As a player,** I want to see credits at the bottom: "Created by Aaron Wilson".

### Animated Atmosphere

- **As a player,** I want the main menu background to be animated with a starry parallax effect.
- **As a developer,** I want to implement the color palettes defined in the design document (e.g., Cyber-Magic for mana-infused tech feel).

### Game Persistence

- **As a player,** I want the "Load Game" option to be available only if a save file (`savegame.json`) actually exists.
- **As a player,** I want to exit the game cleanly from the main menu.

## Technical Tasks

- Create a `RunState::MainMenu` and ensure the game starts in this state.
- Implement `render_main_menu` in `renderer.rs` using `ratatui` layouts.
- Implement a background animation system for the stars (likely using a dedicated component or state in `App`).
- Update the input handler to process menu selection (Up/Down/Enter).
- Update `persistence.rs` to provide a `has_save_game()` helper.
