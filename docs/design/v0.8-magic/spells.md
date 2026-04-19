# Spells

**Status:** Active Development

## Introduction

We will add *Spells* to the game.

Here is the flow for casting a spell.

### Spell Definition

A spell is defined by the following:

```yaml
Spell:
    title: string
    description: string
    mana_cost:
        orange: int in [0, 5]
        purple: int in [0, 5]
    targeting: TargetSpec
    effects: SpellEffect[]
```

A spell's `level` is a derived value `mana_cost.orange + mana_cost.purple`.

*Note*: The following spells are invalid and must be rejected at load time:
- Both orange and purple costs are non-zero (mixed-color spells are not supported yet).
- Both costs are zero (level-0 free spells are not valid).
