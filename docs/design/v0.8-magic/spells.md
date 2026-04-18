# Spells

**Status:** Active Development

## Changelog

- 2026-18-04: Document created

## Introduction

We will add *Spells* to the game.

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
```

A spell's `level` is a derived value `mana_cost.orange + mana_cost.purple`.

*Note*: For now, spells either have orange or purple mana, not both.

## Content File Spec

```yaml
Spell:
    title: string
    description: string
    mana_cost:
        orange: int in [0, 5]
        purple: int in [0, 5]
    targeting: string - reference to a targetting strategy
    effects: SpellEffect[]
```
