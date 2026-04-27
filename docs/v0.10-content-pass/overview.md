# Epic: v0.10 Content Pass & Content Manifests

This version focuses on maturing the game's data pipeline and expanding the
narrative weight of the dungeon through lore-driven content. By moving from a
single JSON file to a directory of YAML manifests, we enable easier content
creation and better organization for the growing set of monsters, items, and
spells.

## Key Goals

1. **YAML Migration**: Transition from a single, bulky `content.json` to a modular `content/` directory using YAML for better readability.
2. **Zero Hardcoding**: Audit the spawner and level generator to ensure all entities and floor rules are data-driven.
3. **Lore Foundations**: Introduce the lore snippet system for Solari, Nihil, Aetheric, Biomass, and Iron factions.
4. **Look Action**: Implement a dedicated inspection mode (key: 'l') to view entity descriptions and lore.
5. **Content Batch #1**: Add a significant new wave of monsters and items that leverage the mechanics introduced in v0.8 (Magic) and v0.9 (Gunplay).

## Reference Documents

- [Implementation Plan](implementation-plan.md)
- [Roadmap](../roadmap.md)
- [Design: Player](design/mechanics/player.md)
- [Design: Color Palette](design/style/color-palette.md)

## Success Criteria

- Game loads all content from `content/*.yaml`.
- `content.json` is removed.
- Player stats and starting equipment are defined in YAML.
- Tag-based spawning system allows selecting objects by role (e.g., `door`, `trap`).
- "Look" feature (mapped to 'l') allows inspecting all entities in FOV.
- Vim movement keys (`hjkl`) are removed in favor of standard directions.
- At least 20 new lore-integrated entities added.
- `reload_content` debug command functional.
- `--check-content` CLI flag successfully validates the `content/` directory.
- Level generation outputs a debug spawn summary.
- All tests pass (confirming no regression in content loading).
