# Effects

**Status:** Active Development

This document defines primitive game effects. Effects and Status Effects may be applied by any entity — including monsters — via code. However, in v0.8, only the player goes through the full Spell/Cast flow. Monsters do not have Spellbooks or ManaPool components, but monster AI routines may invoke these effect opcodes directly.

--- Content file representation of effects:

```yaml
EffectBase: &EffectBase
    application_save: optional { STR | DEX | CON | INT | WIS | CHA }  # rolled once at cast time to resist initial application
    shape: point | circle   # point = origin only; circle = radius around origin
    radius: optional int    # only relevant when shape is circle

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

# Moves the affected along the vector, legal moves only.
# The semantic meaning of xComponent/yComponent (e.g. away-from-caster, fixed-direction)
# is defined by the specific opcode implementation, not this schema.
Push:
    <<: *EffectBase
    xComponent: int
    yComponent: int

# Instantly moves the affected to the relative location.
# Vector semantics are opcode-defined (see Push note above).
Teleport:
    <<: *EffectBase
    xComponent: int
    yComponent: int

Heal:
    <<: *EffectBase
    magnitude: !dice "1d6+2"

# Create an entity. The target must be a location (point); entity selection targets are not valid for this effect.
CreateEntity:
    <<: *EffectBase
    entityType: string      # string representing the entity to create
```

## Resisting Effects

There are two distinct save contexts in the magic system:

- **Application save** (`application_save` on an effect): rolled once at cast time when the effect first applies. If the target succeeds, the effect does not apply to them at all.
- **Recovery save** (`recovery_save` on a status effect): rolled each turn while the status is active. If the target succeeds, the status is removed early. See `status_effects.md`.

A Spell's Save DC is equal to `10 + Spell Level + Caster's CHA mod`. The Save attribute is specified per-effect.
