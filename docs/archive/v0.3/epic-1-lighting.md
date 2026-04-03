# Epic 1: Dynamic Lighting & Vision

This epic adds atmosphere and tactical depth by making light a critical resource.

## User Stories

### Light Sources
- **As a player,** I want to find and use torches that provide a circular area of bright light around me.
- **As a player,** I want to see glowing crystals or lamps in the dungeon that illuminate specific rooms or corridors.

### Tactical Darkness
- **As a player,** I want monsters to be harder to detect in low light, requiring me to move carefully or use light to reveal them.
- **As a player,** I want my torches to slowly flicker and eventually burn out, forcing me to manage my light resources.

### Developer Goals
- Implement a lighting overlay that modifies the colors of tiles based on light intensity.
- Integrate lighting into the Field of View (FOV) calculation: tiles are only "Visible" if they are both in the Viewshed AND sufficiently lit.
