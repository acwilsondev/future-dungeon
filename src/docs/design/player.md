# Character Model

## Main Attributes

- HP (goes up by a default amount per level)
- Class
    - Starting equipment and/or spells.
    - Starting attributes (15, 14, 13, 12, 10, 8)
    - Classes do NOT provide other static benefits (i.e. a fighter can train to be a wizard over time).

Our first class will be Fighter. It assigns the following attributes:

- STR > CON > DEX > WIS > CHA > INT
- It begins with Chainmail, Longsword, and a Shield.

We will not implement other classes as part of the first character initiative.

## Attributes

Generally speaking we use a DC system for all roles.

Attributes give a standard D&D bonus: int((Score - 10) / 2)

Each attribute generally is associated with a type of "Save", which is a protection against harm.

- STR
    - Heavy melee bonus
    - Heavy melee hit bonus
    - Carry capacity
- DEX
    - Stealthiness
    - Ranged hit bonus
    - Ranged damage bonus
    - Small weapon hit bonus
    - Small weapon damage bonus
    - Trap saves
    - Dodge bonus
    - Capped by heavier armors
- CON
    - bonus HP (per pc level, retroactive)
    - poison saves
- INT
    - % chance to learn scrolls
    - % chance to identify monsters, items
    - Arcane spell potency
- WIS
    - Vision radius
    - Stealth detection
    - Trap and hidden door detection
    - Divine spell potency
- CHA
    - Merchant price improvements
    - Chance to befriend
    - Very small, hidden chance for lucky boons
