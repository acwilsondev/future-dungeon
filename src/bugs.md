# Bugs

- [x] Monsters should not attack Alchemy Stations.
- [x] Dex Mod Cap by heavier armors should apply to all Dex Mod checks
- [x] Light should not be a separate item slot- it should be wielded like a normal wieldable.
- [x] Weapons that only take up one slot should be equippable to either hand.
- [x] Off Hand attack proc chance should depend on the ATTR mod the weapon uses, not always DEX.
- [x] Damage in log is not shown correctly for Critical Hits
  - `CRITICAL HIT! Player hits Spider for 8 damage! (Roll:20+0 vs DC:10, Dmg:4+0 DR:0)`
  - Should be `CRITICAL HIT! Player hits Spider for 8 damage! (Roll:20+0 vs DC:10, Dmg:8+0 DR:0)`
- [x] Critical Hit damage should roll damage twice and use the total, not multiple a single roll by 2. See below:

```rs
// incorrect
damage = (weapon_roll + attr_mod + power_bonus - target_av).max(1);

if critical {
    damage *= 2;
}

// correct formula for critical hit should be 
// max(
//  1, 
//  (weapon_roll_1 + weapon_roll_2 + attr mod + power bonus) - target_av
// )
```

- [x] Game log should show when a creature dies.
- [x] Equipping a range weapon should not require ammunition.
