# Epic 2: Game Persistence

This epic ensures that the game state can be saved and loaded, while adhering to the principle of permadeath.

## User Stories

### Save and Load
- **As a player,** I want to be able to save my current game and exit, so that I can resume my progress later.
- **As a developer,** I want the entire game state (map, player, monsters, items) to be serializable, ensuring consistency upon loading.

### Permadeath Integration
- **As a game designer,** I want the save file to be deleted or invalidated when the player loads the game, so that they cannot "save-scum" and must face the consequences of their actions (Permadeath).
- **As a player,** I want my character to be permanently deleted when I die, so that the stakes of exploration remain high.

### Metadata & High Scores
- **As a player,** I want to see a record of my previous runs (score, cause of death, deepest level reached), so that I can track my improvement over time.
