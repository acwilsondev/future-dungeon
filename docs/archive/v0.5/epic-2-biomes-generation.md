# Epic 2: Biomes & Environmental Variety

This epic introduces themed segments of the dungeon, varying the visual style, generation algorithms, and monster populations.

## User Stories

### Segmented Biomes

- **As a player,** I want the dungeon's theme to change every 10 levels (starting at level 1, 11, 21...), giving me a sense of moving through different regions.
- **As a player,** I want to see different monsters and lighting colors depending on the current biome (e.g., green/dim in Caves, red/bright in Temples).
  - This lighting effect should not effect torches- we should create new light sources that shine new light colors.
  - Notably, this may require doing a light source summation.

### Organic Cave Generation

- **As a player,** I want to explore "Caves" biomes that use organic, non-linear layouts generated via Cellular Automata.
- **As a player,** I want to encounter "Wet Caves" with significant water obstacles.

### Dark & Light Biomes

- Some floors (20%) randomly have no natural light.
- Some floors (20%) have 5x as many natural lights.

### Themed Biomes

- **Crypt:** Focus on undead enemies and narrow, claustrophobic corridors.
- **Temple:** Symmetric layout, colored lighting (using the `LightSource` system), and organized monster squads.
- **Hell:** Deep-level biome with magma hazards and demonic entities.

## Technical Tasks

- Implement a `CellularAutomata` generator for cave systems.
- Update `Map` to support biome-specific tile sets and environmental effects (like water/magma).
- Integrate biome selection into the `DungeonRhythm` system.
- Implement "Dark" and "Bright" floor modifiers that override default lighting.
