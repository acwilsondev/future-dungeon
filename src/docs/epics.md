# Epics Overview: RustLike v0.2 - "The Deep Descent"

This document outlines the high-level epics for the second major iteration of RustLike, focusing on depth, variety, and mechanical complexity.

## 1. Multi-level Dungeon Depth
Implement stairs (up/down) and a system for generating and persisting multiple dungeon levels. Scaled difficulty based on current depth.

## 2. Advanced Map Features
Enhance procedural generation with "vaults" (pre-designed rooms), thematic levels (e.g., caves vs. halls), and interactive terrain (doors, traps).

## 3. Combat 2.0: Ranged & Magic
Introduce ranged weapons (bows/throwing knives) and a magic system (spells/scrolls) with a dedicated targeting sub-state.

## 4. Status Effects & Temporal Mechanics
Implement duration-based buffs and debuffs (e.g., Poison, Haste, Confusion) and a more robust turn-tick system.

## 5. Monster AI 2.0: Personality & Factions
Expand monster behaviors to include fleeing, kiting, and teamwork. Introduce monster factions (e.g., Goblins vs. Undead).

## 6. Experience & Progression
Create an XP system where defeating monsters leads to level-ups, allowing players to choose stat increases or new perks.

## 7. Economy: Wealth & Shops
Introduce Gold as a currency and spawn periodic "shops" where players can buy and sell items through a trade interface.

## 8. Visual Polish & Feedback
Add simple animations for hits, projectiles, and screen shakes to make combat feel more impactful.

## 9. Knowledge Base & Extended UI
Implement a scrollable message history, a monster bestiary, and detailed item tooltips to help players understand the world.

## 10. Data-Driven Content Pipeline
Externalize monster and item definitions into data files (YAML or JSON) to allow for rapid content creation and balancing without recompiling.
