# Epic 2: Environmental Hazard System

This epic introduces reactive "chemistry" to the dungeon, making the environment itself a threat or a tool.

## User Stories

### Fire Propagation
- **As a player,** I want fire (from spells or traps) to spread to adjacent flammable tiles (like wooden floors or oil pools).
- **As a player,** I want to be able to use water potions or find pools of water to extinguish fires in my path.

### Gas & Vapors
- **As a player,** I want to encounter poison gas clouds that expand over time and eventually dissipate, dealing damage if I linger in them.

### Elemental Interaction
- **As a player,** I want to see logical interactions between elements (e.g., fire creating steam when hitting water, or lightning dealing extra damage to targets in water).

## Developer Goals
- Create an "Elemental State" component for tiles or entities to track fire, wetness, or electrification.
- Implement a "Cloud" entity system for expanding/dissipating gas effects.
