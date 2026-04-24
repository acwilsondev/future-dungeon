use crate::app::{App, Branch, EntitySnapshot};
use crate::components::*;
use hecs::World;

impl App {
    pub fn pack_entities(&mut self) {
        self.entities.clear();
        for (id, (render, render_order)) in self.world.query::<(&Renderable, &RenderOrder)>().iter()
        {
            let pos = self.world.get::<&Position>(id).ok().map(|p| *p);
            let name = self.world.get::<&Name>(id).ok().map(|n| (*n).clone());
            let stats = self.world.get::<&CombatStats>(id).ok().map(|s| *s);
            let attributes = self.world.get::<&Attributes>(id).ok().map(|a| *a);
            let potion = self.world.get::<&Potion>(id).ok().map(|p| *p);
            let weapon = self.world.get::<&Weapon>(id).ok().map(|w| *w);
            let armor = self.world.get::<&Armor>(id).ok().map(|a| *a);
            let door = self.world.get::<&Door>(id).ok().map(|d| *d);
            let trap = self.world.get::<&Trap>(id).ok().map(|t| *t);
            let ranged = self.world.get::<&Ranged>(id).ok().map(|r| *r);
            let ranged_weapon = self.world.get::<&RangedWeapon>(id).ok().map(|rw| *rw);
            let aoe = self.world.get::<&AreaOfEffect>(id).ok().map(|a| *a);
            let confusion = self.world.get::<&Confusion>(id).ok().map(|c| *c);
            let poison = self.world.get::<&Poison>(id).ok().map(|p| *p);
            let strength = self.world.get::<&Strength>(id).ok().map(|s| *s);
            let speed = self.world.get::<&Speed>(id).ok().map(|s| *s);
            let faction = self.world.get::<&Faction>(id).ok().map(|f| *f);
            let viewshed = self.world.get::<&Viewshed>(id).ok().map(|v| *v);
            let personality = self.world.get::<&AIPersonality>(id).ok().map(|p| *p);
            let experience = self.world.get::<&Experience>(id).ok().map(|e| *e);
            let perks = self.world.get::<&Perks>(id).ok().map(|p| (*p).clone());
            let alert_state = self.world.get::<&AlertState>(id).ok().map(|a| *a);
            let hearing = self.world.get::<&Hearing>(id).ok().map(|h| *h);
            let boss = self.world.get::<&Boss>(id).ok().map(|b| (*b).clone());
            let light_source = self.world.get::<&LightSource>(id).ok().map(|l| *l);
            let gold = self.world.get::<&Gold>(id).ok().map(|g| *g);
            let item_value = self.world.get::<&ItemValue>(id).ok().map(|v| *v);
            let obfuscated_name = self
                .world
                .get::<&ObfuscatedName>(id)
                .ok()
                .map(|n| (*n).clone());
            let cursed = self.world.get::<&Cursed>(id).ok().map(|c| *c);
            let equippable = self.world.get::<&Equippable>(id).ok().map(|e| *e);
            let equipped = self.world.get::<&Equipped>(id).ok().map(|e| *e);
            let aegis = self.world.get::<&Aegis>(id).ok().map(|a| *a);
            let aegis_drought = self.world.get::<&AegisDrought>(id).ok().map(|a| *a);
            let aegis_boost = self.world.get::<&AegisBoost>(id).ok().map(|a| *a);
            let mana = self.world.get::<&ManaPool>(id).ok().map(|m| *m);
            let heat = self.world.get::<&HeatMeter>(id).ok().map(|h| *h);
            let shredded = self.world.get::<&Shredded>(id).ok().map(|s| *s);
            let item_stack = self.world.get::<&ItemStack>(id).ok().map(|s| *s);

            self.entities.push(EntitySnapshot {
                pos,
                render: *render,
                render_order: *render_order,
                name,
                stats,
                attributes,
                potion,
                weapon,
                armor,
                door,
                trap,
                ranged,
                ranged_weapon,
                aoe,
                confusion,
                poison,
                strength,
                speed,
                faction,
                viewshed,
                personality,
                experience,
                perks,
                alert_state,
                hearing,
                boss,
                light_source,
                gold,
                item_value,
                obfuscated_name,
                cursed,
                equippable,
                equipped,
                aegis,
                aegis_drought,
                aegis_boost,
                mana,
                heat,
                shredded,
                item_stack,
                is_heavy_ammo: self.world.get::<&HeavyAmmo>(id).is_ok(),
                last_hit_by_player: self.world.get::<&LastHitByPlayer>(id).is_ok(),
                is_levitation: self.world.get::<&Levitation>(id).is_ok(),
                is_merchant: self.world.get::<&Merchant>(id).is_ok(),
                ammo: self.world.get::<&Ammunition>(id).is_ok(),
                consumable: self.world.get::<&Consumable>(id).is_ok(),
                in_backpack: self.world.get::<&InBackpack>(id).is_ok(),
                is_player: self.world.get::<&Player>(id).is_ok(),
                is_monster: self.world.get::<&Monster>(id).is_ok(),
                is_wisp: self.world.get::<&Wisp>(id).is_ok(),
                is_partial_cover: self.world.get::<&PartialCover>(id).is_ok(),
                is_item: self.world.get::<&Item>(id).is_ok(),
                is_down_stairs: self.world.get::<&DownStairs>(id).is_ok(),
                is_up_stairs: self.world.get::<&UpStairs>(id).is_ok(),
                destination: self
                    .world
                    .get::<&DownStairs>(id)
                    .ok()
                    .map(|s| s.destination)
                    .or_else(|| self.world.get::<&UpStairs>(id).ok().map(|s| s.destination)),
            });
        }
    }

    fn add_base_components(cb: &mut hecs::EntityBuilder, e: &EntitySnapshot) {
        if let Some(pos) = e.pos {
            cb.add(pos);
        }
        cb.add(e.render);
        cb.add(e.render_order);
        if let Some(ref name) = e.name {
            cb.add(name.clone());
        }
    }

    fn add_combat_and_stat_components(cb: &mut hecs::EntityBuilder, e: &EntitySnapshot) {
        if let Some(stats) = e.stats {
            cb.add(stats);
        }
        if let Some(attributes) = e.attributes {
            cb.add(attributes);
        }
        if let Some(faction) = e.faction {
            cb.add(faction);
        }
        if let Some(viewshed) = e.viewshed {
            cb.add(viewshed);
        }
        if let Some(personality) = e.personality {
            cb.add(personality);
        }
        if let Some(experience) = e.experience {
            cb.add(experience);
        }
        if let Some(perks) = e.perks.clone() {
            cb.add(perks);
        }
        if let Some(alert_state) = e.alert_state {
            cb.add(alert_state);
        }
        if let Some(hearing) = e.hearing {
            cb.add(hearing);
        }
        if let Some(boss) = e.boss.clone() {
            cb.add(boss);
        }
        if let Some(light_source) = e.light_source {
            cb.add(light_source);
        }
        if let Some(aegis) = e.aegis {
            cb.add(aegis);
        }
        if let Some(aegis_drought) = e.aegis_drought {
            cb.add(aegis_drought);
        }
        if let Some(aegis_boost) = e.aegis_boost {
            cb.add(aegis_boost);
        }
        if let Some(mana) = e.mana {
            cb.add(mana);
        }
        if let Some(heat) = e.heat {
            cb.add(heat);
        }
        if let Some(shredded) = e.shredded {
            cb.add(shredded);
        }
        if let Some(stack) = e.item_stack {
            cb.add(stack);
        }
    }

    fn add_item_components(cb: &mut hecs::EntityBuilder, e: &EntitySnapshot) {
        if let Some(potion) = e.potion {
            cb.add(potion);
        }
        if let Some(weapon) = e.weapon {
            cb.add(weapon);
        }
        if let Some(armor) = e.armor {
            cb.add(armor);
        }
        if let Some(ranged) = e.ranged {
            cb.add(ranged);
        }
        if let Some(ranged_weapon) = e.ranged_weapon {
            cb.add(ranged_weapon);
        }
        if let Some(aoe) = e.aoe {
            cb.add(aoe);
        }
        if let Some(confusion) = e.confusion {
            cb.add(confusion);
        }
        if let Some(poison) = e.poison {
            cb.add(poison);
        }
        if let Some(strength) = e.strength {
            cb.add(strength);
        }
        if let Some(speed) = e.speed {
            cb.add(speed);
        }
        if let Some(gold) = e.gold {
            cb.add(gold);
        }
        if let Some(item_value) = e.item_value {
            cb.add(item_value);
        }
        if let Some(obfuscated_name) = e.obfuscated_name.clone() {
            cb.add(obfuscated_name);
        }
        if let Some(cursed) = e.cursed {
            cb.add(cursed);
        }
        if let Some(equippable) = e.equippable {
            cb.add(equippable);
        }
        if let Some(equipped) = e.equipped {
            cb.add(equipped);
        }
    }

    fn add_marker_and_env_components(cb: &mut hecs::EntityBuilder, e: &EntitySnapshot) {
        if let Some(door) = e.door {
            cb.add(door);
        }
        if let Some(trap) = e.trap {
            cb.add(trap);
        }
        if e.last_hit_by_player {
            cb.add(LastHitByPlayer);
        }
        if e.is_levitation {
            cb.add(Levitation);
        }
        if e.is_merchant {
            cb.add(Merchant);
        }
        if e.ammo {
            cb.add(Ammunition);
        }
        if e.is_heavy_ammo {
            cb.add(HeavyAmmo);
        }
        if e.consumable {
            cb.add(Consumable);
        }
        if e.is_player {
            cb.add(Player);
        }
        if e.is_monster {
            cb.add(Monster);
        }
        if e.is_wisp {
            cb.add(Wisp);
        }
        if e.is_partial_cover {
            cb.add(PartialCover);
        }
        if e.is_item {
            cb.add(Item);
        }
        if e.is_down_stairs {
            cb.add(DownStairs {
                destination: e.destination.unwrap_or((0, Branch::Main)),
            });
        }
        if e.is_up_stairs {
            cb.add(UpStairs {
                destination: e.destination.unwrap_or((0, Branch::Main)),
            });
        }
    }

    fn add_components_to_builder(cb: &mut hecs::EntityBuilder, e: &EntitySnapshot) {
        Self::add_base_components(cb, e);
        Self::add_combat_and_stat_components(cb, e);
        Self::add_item_components(cb, e);
        Self::add_marker_and_env_components(cb, e);
    }

    pub fn unpack_entities(&mut self) -> anyhow::Result<()> {
        self.world = World::new();
        let mut player_entity = None;
        let mut in_backpack_markers = Vec::new();

        for e in &self.entities {
            let mut cb = hecs::EntityBuilder::new();
            Self::add_components_to_builder(&mut cb, e);

            let entity = self.world.spawn(cb.build());
            if e.is_player {
                player_entity = Some(entity);
            }
            if e.in_backpack {
                in_backpack_markers.push(entity);
            }
        }

        if let Some(player) = player_entity {
            for id in in_backpack_markers {
                if let Err(e) = self.world.insert_one(id, InBackpack { owner: player }) {
                    log::error!(
                        "Failed to re-insert InBackpack component for entity {:?}: {}",
                        id,
                        e
                    );
                }
            }
        } else {
            return Err(anyhow::anyhow!("Player entity not found in snapshot"));
        }

        self.map.reinitialize_skipped_fields();
        self.update_fov();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::prelude::Color;

    fn setup_test_app() -> App {
        let mut app = App::new_random().expect("content.json must be present for tests");
        app.world = World::new();
        app
    }

    #[test]
    fn test_comprehensive_serialization() {
        let mut app = setup_test_app();

        // 1. Player entity
        let player = app.world.spawn((
            Player,
            Position { x: 1, y: 2 },
            Renderable {
                glyph: '@',
                fg: Color::White,
            },
            RenderOrder::Player,
            Name("Hero".to_string()),
            CombatStats {
                hp: 10,
                max_hp: 20,
                defense: 3,
                power: 4,
            },
            Experience {
                level: 5,
                xp: 100,
                next_level_xp: 200,
                xp_reward: 0,
            },
            Perks {
                traits: vec![Perk::Toughness],
            },
            Gold { amount: 50 },
            Viewshed { visible_tiles: 10 },
            Faction(FactionKind::Player),
            AIPersonality(Personality::Brave),
            LightSource {
                range: 5,
                base_range: 5,
                color: (1, 2, 3),
                remaining_turns: Some(10),
                flicker: true,
            },
        ));
        app.world
            .insert(
                player,
                (
                    Aegis { current: 3, max: 8 },
                    AegisDrought { duration: 4 },
                    AegisBoost {
                        magnitude: 3,
                        duration: 6,
                    },
                    ManaPool {
                        current_orange: 2,
                        max_orange: 3,
                        current_purple: 1,
                        max_purple: 2,
                        regen_cooldown: 4,
                    },
                ),
            )
            .unwrap();

        // 2. Item 1: Potion
        app.world.spawn((
            Item,
            Position { x: 3, y: 4 },
            Renderable {
                glyph: '!',
                fg: Color::Red,
            },
            RenderOrder::Item,
            Name("Uber Potion".to_string()),
            Potion { heal_amount: 50 },
            AreaOfEffect { radius: 3 },
            Confusion { turns: 5 },
            Poison {
                damage: 2,
                turns: 3,
            },
            Strength {
                amount: 5,
                turns: 10,
            },
            Speed { turns: 10 },
            ItemValue { price: 100 },
            ObfuscatedName("Strange Potion".to_string()),
            Consumable,
            InBackpack { owner: player },
        ));

        // 3. Item 2: Cursed Weapon
        app.world.spawn((
            Item,
            Position { x: 10, y: 10 },
            Renderable {
                glyph: '/',
                fg: Color::White,
            },
            RenderOrder::Item,
            Name("Cursed Sword".to_string()),
            Weapon {
                power_bonus: 10,
                weight: WeaponWeight::Heavy,
                damage_n_dice: 2,
                damage_die_type: 6,
                two_handed: true,
            },
            RangedWeapon {
                range: 8,
                range_increment: 12,
                damage_bonus: 10,
                ..Default::default()
            },
            Cursed,
            Equippable {
                slot: EquipmentSlot::MainHand,
            },
            Equipped {
                slot: EquipmentSlot::MainHand,
            },
            InBackpack { owner: player },
        ));

        // 4. Environment
        app.world.spawn((
            Position { x: 5, y: 6 },
            Renderable {
                glyph: '+',
                fg: Color::Gray,
            },
            RenderOrder::Map,
            Door { open: false },
            Trap {
                damage: 10,
                revealed: true,
            },
            DownStairs {
                destination: (2, Branch::Main),
            },
        ));

        // 5. Pack
        app.pack_entities();
        assert!(app.entities.len() >= 4);

        // 6. Unpack into a new app
        let mut app2 = App::new_test(42);
        app2.entities = app.entities.clone();
        app2.unpack_entities().unwrap();

        // 7. Verify components on player
        let mut player_query = app2.world.query::<(
            &Player,
            &Name,
            &CombatStats,
            &Experience,
            &Perks,
            &Gold,
            &Viewshed,
            &LightSource,
        )>();
        let (_, (_, name, stats, exp, perks, gold, viewshed, light)) =
            player_query.iter().next().unwrap();
        assert_eq!(name.0, "Hero");
        assert_eq!(stats.hp, 10);
        assert_eq!(exp.level, 5);
        assert!(perks.traits.contains(&Perk::Toughness));
        assert_eq!(gold.amount, 50);
        assert_eq!(viewshed.visible_tiles, 10);
        assert_eq!(light.remaining_turns, Some(10));

        let mut aegis_query = app2.world.query::<(&Aegis, &AegisDrought, &AegisBoost, &ManaPool)>();
        let (_, (aegis, drought, boost, pool)) = aegis_query.iter().next().unwrap();
        assert_eq!(aegis.current, 3);
        assert_eq!(aegis.max, 8);
        assert_eq!(drought.duration, 4);
        assert_eq!(boost.magnitude, 3);
        assert_eq!(boost.duration, 6);
        assert_eq!(pool.current_orange, 2);
        assert_eq!(pool.regen_cooldown, 4);

        // 8. Verify Potion
        let mut potion_query = app2.world.query::<(
            &Potion,
            &AreaOfEffect,
            &Confusion,
            &Poison,
            &Strength,
            &Speed,
            &ItemValue,
            &ObfuscatedName,
            &Consumable,
        )>();
        let (_, (potion, aoe, _confusion, _poison, _strength, _speed, val, obf, _)) =
            potion_query.iter().next().unwrap();
        assert_eq!(potion.heal_amount, 50);
        assert_eq!(aoe.radius, 3);
        assert_eq!(val.price, 100);
        assert_eq!(obf.0, "Strange Potion");

        // 9. Verify Cursed Weapon
        let mut weapon_query = app2
            .world
            .query::<(&Weapon, &RangedWeapon, &Cursed, &Equippable, &Equipped)>();
        let (_, (weapon, rw, _, equippable, equipped)) = weapon_query.iter().next().unwrap();
        assert_eq!(weapon.power_bonus, 10);
        assert_eq!(rw.damage_bonus, 10);
        assert_eq!(equippable.slot, EquipmentSlot::MainHand);
        assert_eq!(equipped.slot, EquipmentSlot::MainHand);

        // 10. Verify environment
        let mut env_query = app2.world.query::<(&Door, &Trap, &DownStairs)>();
        let (_, (door, trap, stairs)) = env_query.iter().next().unwrap();
        assert!(!door.open);
        assert_eq!(trap.damage, 10);
        assert!(trap.revealed);
        assert_eq!(stairs.destination, (2, Branch::Main));
    }

    #[test]
    fn test_partial_cover_round_trip() {
        let mut app = setup_test_app();
        app.world.spawn((
            Player,
            Position { x: 0, y: 0 },
            Renderable {
                glyph: '@',
                fg: Color::White,
            },
            RenderOrder::Player,
            Name("Hero".to_string()),
        ));
        crate::spawner::spawn_partial_cover(&mut app.world, 7, 3);

        app.pack_entities();
        let mut app2 = App::new_test(42);
        app2.entities = app.entities.clone();
        app2.unpack_entities().unwrap();

        let mut cover_query = app2.world.query::<(&Position, &PartialCover)>();
        let (_, (pos, _)) = cover_query
            .iter()
            .next()
            .expect("PartialCover should survive round-trip");
        assert_eq!((pos.x, pos.y), (7, 3));
    }
}
