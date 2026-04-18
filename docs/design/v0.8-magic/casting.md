# Casting a Spell

The player may open their Abilities screen using the `a` key. The menu itself works similarly to the Inventory. When an ability is selected, it should execute its logic. This will be called `Casting a Spell` for the remainder of this doc. The act of casting a spell will be referred to as a `Cast`.

## Step 1. Mana Check

In order to cast a spell, the caster must have mana in their mana pool covering the `mana_cost` of the spell.

This result is purely functional and binary. If the player does not have the required mana available, the cast is aborted.

## Step 2. Choose Targets

A spell defines its legal targets in its spell definition. For example,

- any one object in the dungeon
- two mobs
- a spot on the floor or wall
- an item in the character's inventory

Targets may define additional qualifiers as needed (and targetting logic may be highly procedural rather than enumerated).

The player may always abort targetting. If they do so, the cast is aborted.

Targetting does not reveal any information about the world. All targets must be visible (or in the player's inventory).

## Step 3. Pay Mana

Once targets have been validated, the required Mana is removed from the player's mana pool. It cannot be refunded beyond this point. No game effects should occur between Choosing Targets and Paying Mana.

## Step 4. Apply Effects

Once the Mana has been paid, the effects of the spell are applied. This is highly procedural and unique to the spell.

Baseline spell effects work in three ways:

- An instantaneous application
- An applied status effect
- A summoned object

Internal spell logic is *always* instantaneous. If there is ongoing mantainance, it is dispatched to the status effect or summoned object logic.

## Step 5. Cleanup

Since the player spent Mana, their Mana Restoration Clock is reset.

Casting a Spell consumes a full turn.
