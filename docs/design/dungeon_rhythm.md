# Dungeon Rhythm

Defines the sequence of special floors. For the purposes of this doc, we have

- Floors ending in 5: Single Room. Merchant, Holy Altar (heals for now)
- Floors divisible by 10: Normal layout. Special Boss mob
- Floors ended in 1: Biome change (applies to floor n,n+1,...n+9)
- Floors divisible by 20: Character reset shrine hidden somewhere (keeping all XP and items)

## Random Chances

These floors have a small chance of showing up.

- Fairy Fountain: IDK yet

## Biomes

- Normal Dungeon
  - Crypt: Undead enemy focus
- Caves: Cellular automata generation, fungus lights, organic decor
  - Wet Caves: more water than normal
- [elemental] Temple: Symmetric and more monster/decor theming. Colored lighting.
- Hell: reserved for late Dungeon Levels. Magma, Demonic monsters.
- Random Modifiers:
  - Dark [biome]: No lights generate.
  - Bright [biome]: Extra lights
