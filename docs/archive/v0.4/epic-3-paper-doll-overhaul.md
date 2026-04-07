# Epic 3: Paper Doll & Equipment Overhaul

This epic expands the equipment system to a full "Paper Doll" model, allowing for 10 distinct equipment slots.

## User Stories

### Expanded Slots

- **As a player,** I want to equip items in my Head, Torso, Main Hand, Off Hand, Ammo, Hands, Feet, Neck, and two Finger slots.
- **As a player,** I want to see my full equipment list in the inventory screen.

### Two-Handed Weapons

- **As a player,** I want to wield powerful two-handed weapons that use both my Main Hand and Off Hand slots.
- **As a player,** I want my STR modifier to be multiplied by 1.5 when I use a two-handed weapon.

### Armor & Shields

- **As a player,** I want my total AV to be the sum of all my equipped armor and shields.
- **As a player,** I want heavier armors to cap my DEX bonus to Dodge DC.

## Developer Goals

- Replace `EquipmentSlot` with a more comprehensive enum.
- Implement logic to handle two-handed weapon conflicts (clearing the off-hand slot).
- Create a system to calculate total AV and Dodge DC from equipped items.
- Support dual-wielding (including shields).
