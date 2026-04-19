# Casting a Spell

The player may open their Abilities screen using the `a` key. The menu itself works similarly to the Inventory. When an ability is selected, it should execute its logic. This will be called `Casting a Spell` for the remainder of this doc. The act of casting a spell will be referred to as a `Cast`.

## Step 1. Mana Check

In order to cast a spell, the caster must have mana in their mana pool covering the `mana_cost` of the spell.

This result is purely functional and binary. If the player does not have the required mana available, the cast is aborted.

## Step 2. Choose Targets

A spell defines its legal targets via its `TargetSpec`. The three target modes are:

- `self` — automatically targets the caster, no player input needed
- `entity` — player cycles through visible entities within range and confirms one
- `location` — player moves a free cursor to any tile within range and confirms

The player may always abort targeting. If they do so, the cast is aborted.

Targeting does not reveal any information about the world. All targets must be within the player's field of view.

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

Casting a Spell consumes a full turn.

```mermaid
graph TD
    Start([Start Cast]) --> Step1{Step 1: Mana Check}
    
    Step1 -- Fail --> Abort1[Cast Aborted]
    Step1 -- Pass --> Step2[Step 2: Choose Targets]
    
    Step2 --> TargetValidation{Targeting Active?}
    TargetValidation -- Aborted --> Abort2[Cast Aborted]
    TargetValidation -- Confirmed --> Step3[Step 3: Pay Mana]
    
    Step3 --> DroughtCheck{Last Mana?\n(total across all colors = 0)}
    DroughtCheck -- Yes --> ApplyDrought["Apply ManaDrought(5) Status"]
    DroughtCheck -- No --> Step4[Step 4: Apply Effects]
    ApplyDrought --> Step4
    
    subgraph "Effect Loop"
    Step4 --> EffectLoop[Process Shape/Radius]
    EffectLoop --> SaveCheck{Target Saves?}
    SaveCheck -- No --> Resolution[Apply Effect]
    SaveCheck -- Yes --> NextEffect[Next Effect in List]
    Resolution --> NextEffect
    end
    
    NextEffect --> Step5[Step 5: Cleanup]
    Step5 --> End([End Turn])
```
