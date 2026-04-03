# Epic 6: The Fighter Class

This epic implements the first playable class, the Fighter, with its unique starting attributes and gear.

## User Stories

### Starting Attributes

- **As a Fighter,** I want my attributes to be STR (15), CON (14), DEX (13), WIS (12), CHA (10), and INT (8).
- **As a Fighter,** I want my starting HP to be `24 + CON mod`.

### Starting Gear

- **As a Fighter,** I want to begin with a Longsword (Heavy), a Shield, and Chainmail (Heavy).
- **As a Fighter,** I want to have my starting gear already equipped.

### Class Choice on Start

- **As a player,** I want to select the "Fighter" class when I start a new game.

## Developer Goals

- Implement a `Class` component to store a character's starting template.
- Create a `CharacterCreation` system to assign starting attributes and equipment based on the selected class.
- Update the game start flow to include a simple class selection step.
