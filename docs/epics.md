# Epics Overview: RustLike v0.4 - "The Hero's Mantle"

This initiative transforms the player character from a collection of simple stats into a complex, D&D-inspired entity with deep mechanical depth.

## 1. Core Attributes & Modifier System
Implement the six primary attributes (STR, DEX, CON, INT, WIS, CHA) and a centralized modifier calculation system. This will replace the simplified `CombatStats`.

## 2. Advanced Combat Resolution (DC System)
Overhaul the combat logic to use a Difficulty Class (DC) system. Attacks will be resolved with `1d20 + modifier` against a target's `10 + DEX` Dodge DC.

## 3. Paper Doll & Equipment Overhaul
Expand the equipment system to include 10 distinct slots. Implement rules for Two-Handed weapons, Dual Wielding (Main/Off Hand), and Armor/Shield AV stacking.

## 4. Ranged Combat & Ammunition
Introduce a dedicated ranged combat flow. This includes a targeting UI (activated by 'f'), range-based accuracy penalties, and the consumption of ammunition.

## 5. Saving Throws & Hazard Interaction
Implement attribute-based saving throws. Environmental hazards, traps, and monster abilities will now trigger saves (e.g., DEX save vs. fire, CON save vs. poison).

## 6. The Fighter Class
Implement the "Fighter" as the first playable class, including starting attributes, equipment (Chainmail, Longsword, Shield), and progression rules.
