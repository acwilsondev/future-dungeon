# Epic 1: Engine & Rendering

This epic focuses on the core game loop and the visual representation of the game world using a terminal-based interface.

## User Stories

### Core Game Loop
- **As a player,** I want the game to wait for my input before advancing time, so that I can take my time to make tactical decisions (Turn-based).
- **As a developer,** I want a clean separation between game logic and rendering, so that I can easily modify how the game looks without breaking how it works.

### Terminal Display
- **As a player,** I want to see a grid-based representation of the dungeon using Unicode characters, so that I can clearly understand the game state.
- **As a player,** I want the game to use a 256-color palette, so that the environment and entities are visually distinct and aesthetically pleasing.
- **As a developer,** I want a robust rendering abstraction that handles window resizing and efficient terminal updates.

### Input Handling
- **As a player,** I want to use standard keyboard controls (e.g., HJKL or Arrow keys) to interact with the game, so that I can play comfortably.
- **As a player,** I want the game to respond immediately to my key presses, so that the experience feels fluid.
