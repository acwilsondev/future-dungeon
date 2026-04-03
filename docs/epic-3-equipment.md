# Epic 3: Advanced Equipment & Paper Doll

This epic expands character customization by moving beyond generic melee and armor bonuses.

## User Stories

### Equipment Slots

- **As a player,** I want to have specific slots for my gear: Head, Torso, Hands, Feet, and two Finger slots.
- **As a player,** I want to see my current equipment clearly in the Inventory screen.

### Unique Artifacts

- **As a player,** I want to find rare "Artifact" items that provide unique effects, such as "Boots of Levitation" (ignore traps) or a "Ring of Regeneration" (heal over time).

### Visual Representation

- **As a player,** I want the glyph or color of my character to change slightly based on the type of armor I have equipped.

## Developer Goals

- Replace the simple `Weapon` and `Armor` components with a unified `Equippable` component and a `Slots` system.
- Implement "Passive Effects" that components can grant while equipped.
