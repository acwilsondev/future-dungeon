# TargetSpec

Spells have a TargetSpec definition. Here is how TargetSpecs are defined in content files.

```yaml
TargetSpec: &TargetSpec
    range: optional int  # range along line of sight in tiles; ignored when selection is self
    selection: entity | self | location
```

## Resolving Targets

How targets are resolved depends on the `selection` type. The `radius` of any area of effect is defined per-effect (see `effects.md`), not on the TargetSpec.

### Entity

In this mode, an entity is selected in the Target flow. It should cycle between visible entities **within `range` tiles** of the caster.

The target flow returns the selected entity as the origin point. Each effect then applies its own `shape` and `radius` to determine which entities are hit.

### Self

This is a special case of `Entity` that automatically selects the caster.

### Location

In this mode, the player moves a free cursor to choose an origin tile (within `range`). Each effect then applies its own `shape` and `radius` to determine which entities/tiles are affected.
