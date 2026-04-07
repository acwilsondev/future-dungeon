# Epic 4: Ranged Combat & Ammunition

This epic overhauls ranged combat to include dedicated targeting, range increments, and ammunition consumption.

## User Stories

### Ranged Weapon Equipment Slot

- **As a player,** I want to equip my ranged weapon to a regular X-Hand weapon slot (Main Hand, Off Hand).

### Targeted Ranged Attacks

- **As a player,** I want to activate my ranged weapon by pressing `f`, if I have a Ranged Weapon equipped.
- **As a player,** I want to select a target using a visual targeting cursor.

### Range-Based Accuracy

- **As a player,** I want ranged weapons to have a Range Increment statistic.
- **As a player,** I want to have a standard attack roll at short range, with penalties (disadvantage) for each full increment of range beyond the weapon's base range.
  - This disadvantage should apply *per* range increment. For example, if a weapon has a range increment of 10 and I aim at  a target at 11 distance, I must roll twice and take the lower result. At 21, I must roll three times and take the lowest, etc.

### Using a Ranged Weapon in Melee

- **As a player, I want to be able to bump attack with a ranged weapon equipped.
  - When doing so, the attack uses lower Improvised Weapon stats.
  - This mode uses STR as the attack and damage mod.

### Ammunition Consumption

- **As a player,** I want to consume ammunition (arrows, bolts) from my Ammo slot when making ranged attacks.
- **As a player,** I want my ranged weapon to be unusable if I have no matching ammunition equipped.

### Cross Wielding

- **As a player,** If I am wielding a melee weapon in my Main Hand and a Ranged Weapon in my Off Hand, I can:
  - Bump attack with the Main Hand weapon and proc an improvised weapon attack with my Off Hand.
  - `f` attack with the off hand weapon.
- **As a player,** If I am wielding a ranged weapon in my Main Hand and a Melee Weapon in my Off Hand, I can:
  - Bump attack with the improvised weapon in my main hand and proc an off hand attack.
  - `f` attack with the off hand weapon.

## Developer Goals

- All current ranged weapons must be two-handed. However, this is not necessarily the case.
  - A Hand Crossbow is one-handed and may be dual wielded.
  - Attacks with dual ranged weapons proc the same as melee weapons.
- Create a `TargetingSystem` to handle the 'f' key flow and cursor movement.
- Implement the range increment logic (rolling 1d20 twice and taking the lower result for each increment).
- Ensure ranged attacks correctly consume items from the `Ammo` slot.