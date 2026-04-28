use crate::components::*;
use crate::content::{RawFeature, RawFeatureKind, RawItem, RawMonster, RawPlayerDefaults};
use hecs::World;
use ratatui::prelude::Color;

pub fn spawn_player(world: &mut World, x: u16, y: u16, d: &RawPlayerDefaults) -> hecs::Entity {
    world.spawn((
        Position { x, y },
        Renderable {
            glyph: '@',
            fg: Color::Yellow,
        },
        RenderOrder::Player,
        Player,
        Faction(FactionKind::Player),
        Viewshed {
            visible_tiles: d.viewshed,
        },
        Hearing {
            range: d.hearing_range,
        },
        LightSource {
            range: d.light_range,
            base_range: d.light_range,
            color: (150, 150, 100),
            remaining_turns: None,
            flicker: false,
        },
        Name("Player".to_string()),
        Attributes {
            strength: d.str,
            dexterity: d.dex,
            constitution: d.con,
            intelligence: d.int,
            wisdom: d.wis,
            charisma: d.cha,
        },
        CombatStats {
            max_hp: d.max_hp,
            hp: d.max_hp,
            defense: d.defense,
            power: d.power,
        },
        Experience {
            level: 1,
            xp: 0,
            next_level_xp: 50,
            xp_reward: 0,
        },
        Perks { traits: Vec::new() },
        Gold { amount: 0 },
        Aegis {
            current: d.aegis,
            max: d.aegis,
        },
    ))
}

pub fn spawn_monster(
    world: &mut World,
    x: u16,
    y: u16,
    raw: &RawMonster,
    dungeon_level: u16,
) -> hecs::Entity {
    let hp = raw.hp + (dungeon_level as i32 * 2);
    let power = raw.power + (dungeon_level as i32 / 2);

    let mut cb = hecs::EntityBuilder::new();
    cb.add(Position { x, y });
    cb.add(Renderable {
        glyph: raw.glyph,
        fg: Color::Rgb(raw.color.0, raw.color.1, raw.color.2),
    });
    cb.add(RenderOrder::Monster);
    cb.add(Monster);
    cb.add(Faction(raw.faction));
    cb.add(AIPersonality(raw.personality));
    cb.add(Viewshed {
        visible_tiles: raw.viewshed,
    });
    cb.add(Hearing { range: 10 });
    cb.add(AlertState::Sleeping);
    cb.add(Name(raw.name.clone()));
    cb.add(Attributes {
        strength: 10,
        dexterity: 10,
        constitution: 10,
        intelligence: 10,
        wisdom: 10,
        charisma: 10,
    });
    cb.add(CombatStats {
        max_hp: hp,
        hp,
        defense: raw.defense,
        power,
    });
    cb.add(Experience {
        level: dungeon_level as i32,
        xp: 0,
        next_level_xp: 0,
        xp_reward: raw.xp_reward + (dungeon_level as i32 * 5),
    });

    if let Some(r) = raw.ranged {
        cb.add(RangedWeapon {
            range: r as i32,
            range_increment: r as i32,
            damage_bonus: power,
            ..Default::default()
        });
    }

    if let Some(t) = raw.confusion {
        cb.add(Confusion { turns: t });
    }
    if let Some((d, t)) = raw.poison {
        cb.add(Poison {
            damage: d,
            turns: t,
        });
    }

    if let Some(true) = raw.is_boss {
        let mut phases = Vec::new();
        if let Some(raw_phases) = &raw.phases {
            for rp in raw_phases {
                phases.push(BossPhase {
                    hp_threshold: (hp as f32 * rp.hp_threshold_pct) as i32,
                    action: rp.action,
                    triggered: false,
                });
            }
        }
        cb.add(Boss { phases });
    }

    world.spawn(cb.build())
}

fn add_item_components(cb: &mut hecs::EntityBuilder, raw: &RawItem) {
    cb.add(Renderable {
        glyph: raw.glyph,
        fg: Color::Rgb(raw.color.0, raw.color.1, raw.color.2),
    });
    cb.add(RenderOrder::Item);
    cb.add(Item);
    cb.add(Name(raw.name.clone()));
    cb.add(ItemValue { price: raw.price });

    if let Some(obf) = &raw.obfuscated_name {
        cb.add(ObfuscatedName(obf.clone()));
    }
    if let Some(true) = raw.cursed {
        cb.add(Cursed);
    }
    if let Some(slot) = raw.slot {
        cb.add(Equippable { slot });
    }
    if let Some(h) = raw.potion {
        cb.add(Potion { heal_amount: h });
    }
    if let Some(w) = &raw.weapon {
        cb.add(Weapon {
            power_bonus: w.power_bonus,
            weight: w.weight,
            damage_n_dice: w.n_dice,
            damage_die_type: w.die_type,
            two_handed: w.two_handed,
        });
    }
    if let Some(a) = &raw.armor {
        cb.add(Armor {
            defense_bonus: a.defense_bonus,
            max_dex_bonus: a.max_dex_bonus,
        });
    }
    if let Some(r) = raw.ranged {
        cb.add(Ranged { range: r });
    }
    if let Some(rw) = &raw.ranged_weapon {
        let power_source = rw.power_source().unwrap_or(WeaponPowerSource::Ammo);
        let heat_per_shot = rw.heat_per_shot.unwrap_or(1);
        let element = rw.element_type().unwrap_or(None);
        cb.add(RangedWeapon {
            range: rw.range,
            range_increment: rw.range_increment,
            damage_bonus: rw.damage_bonus,
            power_source,
            heat_per_shot,
            efficient_cooldown: rw.efficient_cooldown,
            burst_count: rw.burst_count.unwrap_or(1),
            scatter: rw.scatter,
            shredding: rw.shredding,
            tachyonic: rw.tachyonic,
            element,
        });
        if power_source == WeaponPowerSource::Heat {
            let capacity = rw.heat_capacity.unwrap_or(6);
            cb.add(HeatMeter {
                current: 0,
                capacity,
                venting: 0,
            });
        }
    }
    if let Some(r) = raw.aoe {
        cb.add(AreaOfEffect { radius: r });
    }
    if let Some(t) = raw.confusion {
        cb.add(Confusion { turns: t });
    }
    if let Some((d, t)) = raw.poison {
        cb.add(Poison {
            damage: d,
            turns: t,
        });
    }
    if raw.ammo {
        cb.add(Ammunition);
    }
    if raw.heavy_ammo {
        cb.add(HeavyAmmo);
    }
    if let Some(count) = raw.stack {
        cb.add(ItemStack { count });
    }
    if raw.consumable {
        cb.add(Consumable);
    }
    if let Some(light) = &raw.light {
        cb.add(LightSource {
            range: light.range,
            base_range: light.range,
            color: light.color,
            remaining_turns: light.turns,
            flicker: light.flicker,
        });
    }
    if raw.levitation {
        cb.add(Levitation);
    }
    if raw.regeneration {
        cb.add(Regeneration);
    }
}

pub fn spawn_item(world: &mut World, x: u16, y: u16, raw: &RawItem) -> hecs::Entity {
    let mut cb = hecs::EntityBuilder::new();
    cb.add(Position { x, y });
    add_item_components(&mut cb, raw);
    world.spawn(cb.build())
}

pub fn spawn_gold(world: &mut World, x: u16, y: u16, amount: i32) -> hecs::Entity {
    world.spawn((
        Position { x, y },
        Renderable {
            glyph: '*',
            fg: Color::Yellow,
        },
        RenderOrder::Item,
        Name(format!("{} Gold", amount)),
        Gold { amount },
    ))
}

pub fn spawn_stairs(
    world: &mut World,
    x: u16,
    y: u16,
    down: bool,
    destination: (u16, Branch),
) -> hecs::Entity {
    if down {
        world.spawn((
            Position { x, y },
            Renderable {
                glyph: '>',
                fg: Color::White,
            },
            RenderOrder::Map,
            DownStairs { destination },
            Name("Down Stairs".to_string()),
        ))
    } else {
        world.spawn((
            Position { x, y },
            Renderable {
                glyph: '<',
                fg: Color::White,
            },
            RenderOrder::Map,
            UpStairs { destination },
            Name("Up Stairs".to_string()),
        ))
    }
}

pub fn spawn_feature(world: &mut World, x: u16, y: u16, raw: &RawFeature) -> hecs::Entity {
    let fg = Color::Rgb(raw.color.0, raw.color.1, raw.color.2);
    match &raw.kind {
        RawFeatureKind::Door => world.spawn((
            Position { x, y },
            Renderable {
                glyph: raw.glyph,
                fg,
            },
            RenderOrder::Map,
            Door { open: false },
            Name(raw.name.clone()),
        )),
        RawFeatureKind::Trap { damage } => world.spawn((
            Position { x, y },
            Renderable {
                glyph: raw.glyph,
                fg,
            },
            RenderOrder::Trap,
            Trap {
                damage: *damage,
                revealed: false,
            },
            Name(raw.name.clone()),
        )),
        RawFeatureKind::PoisonTrap { damage, turns } => world.spawn((
            Position { x, y },
            Renderable {
                glyph: raw.glyph,
                fg,
            },
            RenderOrder::Trap,
            Trap {
                damage: 0,
                revealed: true,
            },
            Poison {
                damage: *damage,
                turns: *turns,
            },
            Name(raw.name.clone()),
        )),
        RawFeatureKind::Cover => world.spawn((
            Position { x, y },
            Renderable {
                glyph: raw.glyph,
                fg,
            },
            RenderOrder::Map,
            PartialCover,
            Name(raw.name.clone()),
        )),
    }
}

pub fn spawn_door(world: &mut World, x: u16, y: u16) -> hecs::Entity {
    world.spawn((
        Position { x, y },
        Renderable {
            glyph: '+',
            fg: Color::Indexed(94),
        },
        RenderOrder::Map,
        Door { open: false },
        Name("Door".to_string()),
    ))
}

pub fn spawn_partial_cover(world: &mut World, x: u16, y: u16) -> hecs::Entity {
    world.spawn((
        Position { x, y },
        Renderable {
            glyph: '.',
            fg: Color::Rgb(150, 120, 80),
        },
        RenderOrder::Map,
        PartialCover,
        Name("Debris".to_string()),
    ))
}

pub fn spawn_trap(world: &mut World, x: u16, y: u16) -> hecs::Entity {
    world.spawn((
        Position { x, y },
        Renderable {
            glyph: '^',
            fg: Color::Red,
        },
        RenderOrder::Trap,
        Trap {
            damage: 5,
            revealed: false,
        },
        Name("Trap".to_string()),
    ))
}

pub fn spawn_merchant(world: &mut World, x: u16, y: u16) -> hecs::Entity {
    world.spawn((
        Position { x, y },
        Renderable {
            glyph: 'M',
            fg: Color::Rgb(255, 165, 0),
        },
        RenderOrder::Monster,
        Merchant,
        Name("Merchant".to_string()),
        CombatStats {
            max_hp: 100,
            hp: 100,
            defense: 10,
            power: 10,
        },
        Faction(FactionKind::Player),
        Viewshed { visible_tiles: 8 },
        Hearing { range: 10 },
        AlertState::Aggressive,
        AIPersonality(Personality::Tactical),
    ))
}

pub fn spawn_light_crystal(world: &mut World, x: u16, y: u16, color: Color) -> hecs::Entity {
    let rgb = match color {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (100, 149, 237), // Fallback
    };
    world.spawn((
        Position { x, y },
        Renderable {
            glyph: '*',
            fg: color,
        },
        RenderOrder::Map,
        LightSource {
            range: 4,
            base_range: 4,
            color: rgb,
            remaining_turns: None,
            flicker: true,
        },
        Name("Glowing Crystal".to_string()),
    ))
}

pub fn spawn_wisp(world: &mut World, x: u16, y: u16) -> hecs::Entity {
    world.spawn((
        Position { x, y },
        Renderable {
            glyph: '*',
            fg: Color::Cyan,
        },
        RenderOrder::Map,
        LightSource {
            range: 4,
            base_range: 4,
            color: (0, 255, 255),
            remaining_turns: None,
            flicker: true,
        },
        Wisp,
        Name("Dungeon Wisp".to_string()),
    ))
}

pub fn spawn_alchemy_station(world: &mut World, x: u16, y: u16) -> hecs::Entity {
    world.spawn((
        Position { x, y },
        Renderable {
            glyph: 'A',
            fg: Color::Rgb(200, 100, 200),
        },
        RenderOrder::Map,
        AlchemyStation,
        Name("Alchemy Station".to_string()),
    ))
}

pub fn spawn_holy_altar(world: &mut World, x: u16, y: u16) -> hecs::Entity {
    world.spawn((
        Position { x, y },
        Renderable {
            glyph: 'T',
            fg: Color::Rgb(255, 255, 255),
        },
        RenderOrder::Map,
        HolyAltar,
        Name("Holy Altar".to_string()),
    ))
}

pub fn spawn_reset_shrine(world: &mut World, x: u16, y: u16) -> hecs::Entity {
    world.spawn((
        Position { x, y },
        Renderable {
            glyph: 'S',
            fg: Color::Rgb(0, 255, 0),
        },
        RenderOrder::Map,
        ResetShrine,
        Name("Reset Shrine".to_string()),
    ))
}

pub fn spawn_mana_shrine(world: &mut World, x: u16, y: u16, color: ManaColor) -> hecs::Entity {
    let (r, g, b) = match color {
        ManaColor::Orange => (255, 165, 0),
        ManaColor::Purple => (160, 90, 200),
    };
    let name = format!("{} Shrine", color.order_name());
    world.spawn((
        Position { x, y },
        Renderable {
            glyph: '&',
            fg: Color::Rgb(r, g, b),
        },
        RenderOrder::Map,
        Shrine {
            color,
            tried: false,
        },
        Name(name),
    ))
}

pub fn spawn_tome(
    world: &mut World,
    x: u16,
    y: u16,
    spell_name: &str,
    color: ManaColor,
    level: u32,
) -> hecs::Entity {
    let (r, g, b) = match color {
        ManaColor::Orange => (255, 165, 0),
        ManaColor::Purple => (160, 90, 200),
    };
    let real_name = format!("Tome of {}", spell_name);
    let obfuscated = format!("Strange {} Tome", color.order_name());
    world.spawn((
        Position { x, y },
        Renderable {
            glyph: '=',
            fg: Color::Rgb(r, g, b),
        },
        RenderOrder::Item,
        Item,
        Tome {
            spell_name: spell_name.to_string(),
            color,
            level,
        },
        Name(real_name),
        ObfuscatedName(obfuscated),
    ))
}

pub fn spawn_item_in_backpack(
    world: &mut World,
    owner: hecs::Entity,
    raw: &RawItem,
) -> hecs::Entity {
    let mut cb = hecs::EntityBuilder::new();
    cb.add(InBackpack { owner });
    add_item_components(&mut cb, raw);
    world.spawn(cb.build())
}
