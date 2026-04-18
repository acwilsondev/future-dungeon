# Mana

**Status:** Active Development

Players will gain a new resource called *Mana*. The unit of mana is *Mana* (i.e. 1 Mana, 2 Mana). Mana can have a color (1 Orange Mana, 2 Purple Mana). These values are pools with maximum and current values. For instance, a character may have 1 Orange Mana out of 2 Maximum Orange Mana.

The player's Mana Pool is always capped at 5. If the player attempts to raise their total Mana above 5, the process should fail.

## Recovering Mana

A character's current mana increases by 1 per turn, unless they are under a `ManaDrought` status effect, in which case no mana is recovered that turn. Which color is granted is chosen randomly among the colors with the greatest deficit (max minus current).

For example,

- if I have 1/3 Orange Mana and 1/2 Purple Mana, Orange has a deficit of 2 and Purple has a deficit of 1, so Orange is recovered.
- if I have 1/2 Orange Mana and 1/2 Purple Mana, both have equal deficit, so one is chosen at random.

When a player spends their last Mana — meaning total current mana across all colors reaches zero — they gain a `ManaDrought(5)` status effect. See [status_effects.md](status_effects.md).

## Visualizing Mana

If the player has no max mana (both `max_orange` and `max_purple` are 0), the mana section of the Character Pane is hidden entirely.

Once the player has at least one max mana, it is visualized as 1-5 pips `*`. Pips are ordered as follows:

Orange Unspent, Purple Unspent, Orange Spent, Purple Spent

Spent pips (max − current) are dulled out but should still be visible.
