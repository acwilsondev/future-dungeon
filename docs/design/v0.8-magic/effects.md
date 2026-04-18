# Effects

**Status:** Active Development

This document defines primitive game effects.

Content file representation of effects

```yaml
EffectBase: &EffectBase
    save: optional { STR | DEX | CON | INT | WIS | CHA }

# Applies a given status effect to the affected
GrantStatus:
    <<: *EffectBase
    status: *StatusEffect

# Removes a status effect of the given type to the effected
RemoveStatus:
    <<: *EffectBase
    statusType: string      # string representing the status to remove

# DealDamage always flows through AV and resistance checks.
# This represents a resolved hit (whether one was rolled or not).
DealDamage:
    <<: *EffectBase
    damageType: Fire | Poison | Bludgeoning | Slashing | Piercing
    # damage roll given as [count]d[die]+[bonus]
    magnitude: !dice "1d6+2"

# Moves the affected along the vector, legal moves only
Push:
    <<: *EffectBase
    xComponent: int
    yComponent: int

# Instantly moves the affected to the relative location
Teleport:
    <<: *EffectBase
    xComponent: int
    yComponent: int

# Permanent additive alteration to an attribute
ModifyAttribute:
    <<: *EffectBase
    attribute: optional { STR | DEX | CON | INT | WIS | CHA }
    magnitude: !dice "1d6+2"

Heal:
    <<: *EffectBase
    magnitude: !dice "1d6+2"

# Create an entity. Target is usually a location where the CreateEntity should be created.
CreateEntity:
    <<: *EffectBase
    entityType: string      # string representing the entity to create

```

## Resisting Effects

Spell effects can be resisted by rolling a Save. A Spell's Save DC is equal to `10 + Spell Level + Caster's CHA mod`. The Save type is determined by the spell effect.
