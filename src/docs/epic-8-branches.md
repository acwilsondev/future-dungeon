# Epic 8: Thematic Dungeon Branches

This epic increases variety by introducing distinct sub-areas with unique challenges.

## User Stories

### Distinct Areas
- **As a player,** I want to encounter "Branches" of the dungeon with unique wall and floor tiles (e.g., "The Overgrown Gardens," "The Frozen Vaults").
- **As a player,** I want each branch to have its own pool of monsters and items that fit the theme.

### Branching Paths
- **As a player,** I want to occasionally choose between two different stairs, leading to different branches with varying difficulties.

### Environmental Mechanics
- **As a player,** I want branches to have unique environmental rules (e.g., the Frozen Vaults make me move slower, the Gardens have patches of poison spores).

## Developer Goals
- Refactor the `MapBuilder` to support multiple "Thematic Generators."
- Implement a "Dungeon Graph" to track how different levels and branches connect.
