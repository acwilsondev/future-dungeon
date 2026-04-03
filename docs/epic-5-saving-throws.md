# Epic 5: Saving Throws & Hazard Interaction

This epic introduces attribute-based saving throws to protect against environmental hazards and status effects.

## User Stories

### Six Saving Throws

- **As a player,** I want to have a STR, DEX, CON, INT, WIS, and CHA save based on my corresponding attribute modifier.
- **As a player,** I want to make a save whenever I'm exposed to an effect that allows it.

### Traps and Fire Saves

- **As a player,** I want to make a DEX save to avoid or reduce damage from fireballs, falling tiles, and arrow traps.

### Poison and Gas Saves

- **As a player,** I want to make a CON save to avoid being poisoned by toxic gas or monster bites.

### Status Effect Saves

- **As a player,** I want to make WIS saves to avoid fear and illusions, and INT saves to avoid confusion.

## Developer Goals

- Implement a `SavingThrow` system that handles 1d20 rolls against a fixed DC for various hazards.
- Update traps and status effects to trigger saving throws before applying their effects.
- Add support for "Half Damage on Save" for certain hazards (like fireballs).
