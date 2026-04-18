# Magic

**Status:** Approved.

This document defines how magic works in this game.

## Changelog

2026-04-17: Replacement of wizard class with general magic.

## Note on Origins

Mana is inherently linked to mystic orders.

- Order of the Solari: Orange
- Order of the Nihil: Purple

## Spells

We will add *Spells* to the game. The player may open their Abilities screen using the `a` key. The menu itself works similarly to the Inventory. When an ability is selected, it should execute its logic. This will be called `Casting a Spell` for the remainder of this doc. The act of casting a spell will be referred to as a `Cast`.

Here is the flow for casting a spell.

### Spell Definition

A spell is defined by the following:

```yaml
Spell:
    title: string
    mana_cost:
        orange: int in [0, 5]
        purple: int in [0, 5]
    targeting: string - reference to a targetting strategy
    effects: SpellEffect[]

SpellEffect:
    type: string reference to the spell effect logic
    // Effect definition contains two parameters passed to the logic.
    x: int in [0, 100]
    y: int in [0, 100]
```

A spell's `level` is a derived value `mana_cost.orange + mana_cost.purple`.

*Note*: For now, spells either have orange or purple mana, not both.

### Casting a Spell

#### Step 1. Mana Check

In order to cast a spell, the caster must have mana in their mana pool covering the `mana_cost` of the spell.

This result is purely functional and binary. If the player does not have the required mana available, the cast is aborted.

#### Step 2. Choose Targets

A spell defines its legal targets in its spell definition. For example,

- any one object in the dungeon
- two mobs
- a spot on the floor or wall
- an item in the character's inventory

Targets may define additional qualifiers as needed (and targetting logic may be highly procedural rather than enumerated).

The player may always abort targetting. If they do so, the cast is aborted.

Targetting does not reveal any information about the world. All targets must be visible (or in the player's inventory).

#### Step 3. Pay Mana

Once targets have been validated, the required Mana is removed from the player's mana pool. It cannot be refunded beyond this point. No game effects should occur between Choosing Targets and Paying Mana.

#### Step 4. Apply Effects

Once the Mana has been paid, the effects of the spell are applied. This is highly procedural and unique to the spell.

#### Step 5. Cleanup

Since the player spent Mana, their Mana Restoration Clock is reset.

Casting a Spell consumes a full turn.

### Resisting Effects

Spell effects can be resisted by rolling a Save. A Spell's Save DC is equal to `10 + Spell Level + Caster's CHA mod`. The Save type is determined by the spell effect.

## Tomes

There are currently usable Scrolls in the game. These items will be replaced by *Tomes* `[`. A Tome is a religious manual for controlling mana, i.e. casting a spell. Tomes are either *Nihil* or *Solari* in origin. Tomes contain instructions for a specific Spell.

An unidentified Tome is titled `Unidentified Solari Tome` or `Unidentified Nihil Tome`. Tomes use the standard Magic Item Identification flow. A Tome's Identification DC is set to `10 + Spell Level`.

Tomes are valuable objects with base price and rarity scaling by Spell Level.

### Studying a Tome

A player may Study a Tome to learn its Spell. To do so, they roll a CHA Check.

`5 + 2 * (Spell Level)` -> Learn the Spell

On a success, the player is prompted `You learned [spell name].`

On a failure, the prompt is `You failed to understand the Tome, and it crumbles.` The Tome is lost.

## Mana

Players will gain a new resource called *Mana*. The unit of mana is *Mana* (i.e. 1 Mana, 2 Mana). Mana can have a color (1 Orange Mana, 2 Purple Mana). These values are pools with maximum and current values. For instance, a character may have 1 Orange Mana out of 2 Maximum Orange Mana.

The player's Mana Pool is always capped at 5. If the player attempts to raise their total Mana above 5, the process should fail.

### Recovering Mana

A character's current mana increases by 1 per turn. Which color is granted is always chosen randomly among the colors with the greatest amount missing.

For example,

- if I have 1/3 Orange Mana and 1/2 Purple Mana, I will recover an Orange Mana next
- if I have 1/2 Orange Mana, 1/2 Purple Mana, I will recover one Mana at random.

When a player spends their last Mana, they gain a `Drought<Mana>(5)` status effect. See [status_effects.md](status_effects.md).

### Visulizing Mana

Once the player has mana, it is visualized in the Character Pane as 1-5 pips `*`. These pips are ordered as follows:

Orange Unspent, Purple Unspent, Orange Spent, Purple Spent

Spent pips are dulled out but should still be visible.

## Shrines

Shrines appear as `&` colored as their origin.

Players may meditate at *Shrines* to attempt to raise their Mana Pool by one Mana.

1. Has this shrine been attempted before? If so, fail.
2. Does this character have five mana already? If so, fail.

Otherwise, they make a CHA check equal to

`10 + (Total Maximum Mana of All Colors)`

On failure, they recieve a message `The shrine is silent. Peace be with you.`

On success, they recieve a message `The shrine resonates with mystic energy. Raise your [Color] mana by one (1) point?`

The color depends on the origin of Shrine.
