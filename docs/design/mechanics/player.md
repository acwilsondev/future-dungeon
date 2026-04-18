# Character Model

Head
    Examples: Leather Cap, Full Helm, Circlet.
    Primary Stat: AV (Armor Value).
Torso (The primary source of AV and DEX caps)
    Examples: Chainmail, Leather Jerkin, Plate Armor.
    Primary Stat: AV / Max DEX Bonus. Later magic items form core build components.
Main Hand
    Valid Items: Wieldables.
    Primary Stat: Weapon Damage / [ATR] Modifier.
Off Hand
    Valid Items: Wieldables.
    Primary Stat: AV (Shield) or Parry Bonus.
Ammo
    Valid Items: Arrows, Bolts
    Note: Consumed by ranged weapons.
Hands
    Examples: Gauntlets, Gloves of Thievery.
    Primary Stat: specific [ATR] buffs (e.g., +1 STR).
Feet
    Examples: Iron Boots, Stealth Boots.
    Primary Stat: AV or Movement/Stealth bonuses.
Neck
    Valid Items: Amulets, Charms, etc
    Primary Stat: Magic effects.
Finger (Left)
    Valid Items: Rings.
    Primary Stat: Magic effects.
Finger (Right)
    Valid Items: Rings.
    Primary Stat: Magic effects.

## `Wielding` Items

Most items can be wielded in the main/off hand. Thus, most items need a (heavy/medium/light)/damage/atr as part of their definition. Two-handed wieldables always take up both hands and cannot be equipped if there is an item in either hand.

Yes, this means you can dual wield shields. A torch in the Off-Hand has a chance of making an Off-Hand attack!

Ranged weapons are Wielded. However, they have basic "improvised weapon" stats when used as a melee weapon via the Bump Action. Wielding a ranged weapon gives a static action that can be activated by pressing `f`. This causes a targetting flow and performs an attack at range.

## Main Attributes

- HP
  - Begins as 24 + CON mod
  - Goes by 8 per level
  - All creatures get a static CON mod * level bonus to HP.
- Class
  - Starting equipment and/or spells.
  - Starting attributes are (15, 14, 13, 12, 10, 8)
  - Leveling up grants +1 to an attribute.
  - Classes do NOT provide other static benefits (i.e. a fighter can train to be a wizard over time).

Our first class will be Fighter. It assigns the following attributes:

- STR > CON > DEX > WIS > CHA > INT
- It begins with Chainmail, Longsword, and a Shield.

We will not implement other classes as part of the first character initiative.

## Attributes

Generally speaking we use a DC system for all rolls.

Attributes give a standard D&D bonus: int((Score - 10) / 2). I think instead of Skills and Proficiency bonus, this number will be unbounded.

Each attribute generally is associated with a type of "Save", which is a protection against harm.

- STR
  - Heavy melee bonus (weapon dmg + str bonus)
  - Heavy melee hit bonus
  - Carry capacity
  - SAVES:
    - Being pushed back
    - pinned by a trap
    - or grappled by a "Grappler" type enemy.
- DEX
  - Stealthiness
  - Ranged hit bonus
  - Ranged damage bonus
  - Small weapon hit bonus
  - Small weapon damage bonus
  - Trap saves
  - Dodge bonus
  - Capped by heavier armors
  - SAVES:
    - Fireballs
    - falling ceiling tiles
    - arrow traps
- CON
  - bonus HP (per pc level, retroactive)
  - poison saves
  - SAVES:
    - Poison gas
    - stunning blows
    - rot
- INT
  - % chance to learn scrolls
  - % chance to identify monsters, items
  - Arcane spell potency
  - SAVES (uncommon):
    - Confusion status effects
    - "Mind Blast" spells
- WIS
  - Vision radius
  - Stealth detection
  - Trap and hidden door detection
  - Divine spell potency
  - SAVES (common):
    - Fear effects
    - magical illusions that hide enemies
- CHA
  - Merchant price improvements
  - Base encounter mood effect
  - Very small, hidden chance for lucky boons
  - SAVES (uncommon):
    - Banishment
    - "Charm" effects that force the player to move randomly
    - Forced polymorph

### Other Attributes

- Armor: Determined by summing the Armor values of equipped items.
- Dodge: Determined by 10 + DEX

## Combat

### Triggering Attacks

An attack is an action a creature can take on its turn. To make a melee attack, the active party bumps into its target.

When this occurs, the creature makes an attack with any weapon equipped in it's Main Hand. If that weapon is Two-Handed, the attack is done.

Then, there is a chance for the character to make an attack with a weapon equipped in its Off-Hand. That chance is determined by [ATRMod]*5%. If the weapon is *light* this chance improves to [ATR Mod]*10%.

### Main Hand Attack & General Attack Resolution

This assumes the character has one weapon equipped in its main hand.

An attack is an action a creature can take on its turn. To make a melee attack, the active party bumps into its target. Then, it rolls `to-hit` 1d20 + [ATR] mod vs the target's Dodge DC (10 + DEX mod). If the result is greater than or equal to the DC, there is a hit.

- hit is given as (1d20 + [ATR mod] >= [target ])
  - A roll of 1 is a Critical Miss. It always misses.
  - A roll of 20 is a Critical Hit. It always hits, and damage is rolled twice (taking the sum).
- damage is given as max(1, ([attacker weapon damage roll] + [ATR mod]) - [target AV])

[ATR] is determined by the `weight` (heavy | medium | light) of the weapon - most melee weapons use STR, while `light` weapons use DEX. Two handed melee weapons apply 1.5x the STR mod (all two handed melee weapons are not light)

### Two Handed Weapons

- The damage roll of a two handed weapon receives 1.5x STR mod (rounded down).

### Off-Hand Attacks

- The Off-Hand attack does not recieve a damage bonus from ATR.

### Ranged Attacks

Ranged attacks are resolved the same, but are activated by pressing F and choosing a target instead of bumping. A ranged attack can only be performed while a ranged weapon is Wielded. Each ranged weapon has a `range` number. For each full increment of `range` the target is away, the `to-hit` roll is rolled again, with the lower value taken.

### Armor and Shields

- Equipped armor provides a static bonus to AV (which reduces damage)
- Shields can be Wielded to provide a static bonus to AV.
- A player *can* equip a shield in both hands.
