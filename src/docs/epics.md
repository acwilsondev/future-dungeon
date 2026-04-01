# Epics Overview: RustLike

This document outlines the high-level epics for the development of "RustLike", a simple Unicode/256-color dungeon crawler roguelike.

## 1. Engine & Rendering
Implement the core game loop and a terminal-based rendering system capable of displaying Unicode characters and 256-color palettes.

## 2. Game Persistence
Implement saving and loading functionality to allow players to resume their progress between sessions. Baking this in early ensures the game state is serializable from the start.

## 3. Procedural Dungeon Generation
Develop algorithms to generate varied and interesting dungeon layouts, including rooms, corridors, and multi-level transitions (stairs).

## 4. Entity Component System (ECS)
Establish a robust system for managing game entities (player, monsters, items) and their behaviors/properties.

## 5. Movement & Exploration
Implement turn-based movement, field-of-view (FOV) calculations, and a "fog of war" mechanic to encourage exploration.

## 6. Combat & Stats System
Create a turn-based combat system with attributes (HP, Strength, etc.), damage calculations, and experience/leveling mechanics.

## 7. Monster AI
Implement basic to intermediate AI for different monster types, ranging from simple "chase the player" to more complex behaviors.

## 8. Item & Inventory System
Design a system for discovering, picking up, and using items, including consumables (potions/food) and equippable gear (weapons/armor).

## 9. User Interface (UI)
Build an in-game UI that displays the map, a scrolling message log for events, and a status sidebar for player information.
