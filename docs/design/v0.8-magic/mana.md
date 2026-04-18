# Mana

Players will gain a new resource called *Mana*. The unit of mana is *Mana* (i.e. 1 Mana, 2 Mana). Mana can have a color (1 Orange Mana, 2 Purple Mana). These values are pools with maximum and current values. For instance, a character may have 1 Orange Mana out of 2 Maximum Orange Mana.

The player's Mana Pool is always capped at 5. If the player attempts to raise their total Mana above 5, the process should fail.

## Recovering Mana

A character's current mana increases by 1 per turn. Which color is granted is always chosen randomly among the colors with the greatest amount missing.

For example,

- if I have 1/3 Orange Mana and 1/2 Purple Mana, I will recover an Orange Mana next
- if I have 1/2 Orange Mana, 1/2 Purple Mana, I will recover one Mana at random.

When a player spends their last Mana, they gain a `Drought<Mana>(5)` status effect. See [status_effects.md](status_effects.md).

## Visulizing Mana

Once the player has mana, it is visualized in the Character Pane as 1-5 pips `*`. These pips are ordered as follows:

Orange Unspent, Purple Unspent, Orange Spent, Purple Spent

Spent pips are dulled out but should still be visible.
