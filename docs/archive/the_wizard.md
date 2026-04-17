# The Wizard

In this epic, the player may begin the game as a Wizard.

## Changes to Scrolls

Using a scroll now has a DC. This DC will be equivalent to 2*(The Spell Level). For now, these checks will be an INT roll. On a success, the scroll is used as normal. On a failure, the scroll is consumed but not applied.

## Heat

Characters gain a new stock called Heat. It is in the range \[0, inf\).

## Changes to the Inventory Menu

The Inventory must be updated to allow for subactions on each item. For a start, we'll allow the user to `Learn Scroll`. This roll will be equivalent to 5+2*(The Spell Level).

## Casting Spells

This epic will add a concept of *Special Abilities*. To begin, these will be activated abilities. The player may open their abilities screen using the `a` key. The menu itself works similarly to the Inventory. When an ability is selected, it should behave as if the respective spell scroll or potion has been used. This will be called `Casting a Spell` for the remainder of this doc.

Attempting to cast a spell increases the character's Heat by the spell's level. Then, they make an INT roll with DC equal to their current Heat. If they fail this roll, their Heat resets to 0 and a Warp occurs.

## Warps

A Warp is a complication that occurs. For now, we'll create an Arcane Enforcer that teleports into the level and begins hunting the player.

### Activated Abilities

