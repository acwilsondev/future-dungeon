# Aegis

**Status:** Active Development

*Aegis* is a new stock. It functions as a damage soak.

Aegis recharges much faster than Health heals. When your Aegis is damaged, you gain [AegisDrought(5)](status_effects.md). When your Aegis is depleted, you gain AegisDrought(10).

Otherwise, Aegis recovers 1 point per turn.

Melee attacks (including improvised bump attacking with a ranged weapon) bypass Aegis entirely.

## New Status Effects

| Status Effect  | Type   | Description                                     |
| :------------- | :----- | :---------------------------------------------- |
| `AegisBoost`   | Buff   | Grants **M** temporary `Aegis` for **D** turns. |
| `AegisDrought` | Debuff | Cannot recover `Aegis` for **D** turns.         |

## Aegis GUI

Aegis is displayed in the same bar as Health, but in a cyan color. The bar length is equal to the total of Health and Aegis, with the Aegis portion in Cyan and the Health Portion in the current color.
