# Aegis

**Status:** Active Development

*Aegis* is a technology or spell effect that grants fast-recharging hitpoints above your natural hitpoints. It functions as a damage soak.

Aegis recharges much faster than Health heals. When your Aegis is damaged, you gain [AegisDrought(5)](status_effects.md). When your Aegis is depleted, you gain AegisDrought(10).

Otherwise, Aegis recovers 1 point per turn.

## New Status Effects

| Status Effect  | Type   | Description                                     |
| :------------- | :----- | :---------------------------------------------- |
| `AegisBoost`   | Buff   | Grants **M** temporary `Aegis` for **D** turns. |
| `AegisDrought` | Debuff | Cannot recover `Aegis` for **D** turns.         |
