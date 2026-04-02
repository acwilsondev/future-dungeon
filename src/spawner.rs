use hecs::World;
use crate::components::*;
use crate::content::{RawItem, RawMonster};
use ratatui::prelude::Color;

pub fn spawn_player(world: &mut World, x: u16, y: u16) -> hecs::Entity {
    world.spawn((
        Position { x, y },
        Renderable { glyph: '@', fg: Color::Yellow },
        RenderOrder::Player,
        Player,
        Faction(FactionKind::Player),
        Viewshed { visible_tiles: 8 },
        LightSource { range: 6, color: (255, 255, 200) },
        Name("Player".to_string()),
        CombatStats { max_hp: 30, hp: 30, defense: 2, power: 5 },
        Experience { level: 1, xp: 0, next_level_xp: 50, xp_reward: 0 },
        Perks { traits: Vec::new() },
        Gold { amount: 0 },
    ))
}

pub fn spawn_monster(world: &mut World, x: u16, y: u16, raw: &RawMonster, dungeon_level: u16) -> hecs::Entity {
    let hp = raw.hp + (dungeon_level as i32 * 2);
    let power = raw.power + (dungeon_level as i32 / 2);
    
    let mut cb = hecs::EntityBuilder::new();
    cb.add(Position { x, y });
    cb.add(Renderable { glyph: raw.glyph, fg: Color::Rgb(raw.color.0, raw.color.1, raw.color.2) });
    cb.add(RenderOrder::Monster);
    cb.add(Monster);
    cb.add(Faction(raw.faction));
    cb.add(AIPersonality(raw.personality));
    cb.add(Viewshed { visible_tiles: raw.viewshed });
    cb.add(Name(raw.name.clone()));
    cb.add(CombatStats { max_hp: hp, hp, defense: raw.defense, power });
    cb.add(Experience { level: dungeon_level as i32, xp: 0, next_level_xp: 0, xp_reward: raw.xp_reward + (dungeon_level as i32 * 5) });
    
    if let Some(r) = raw.ranged {
        cb.add(RangedWeapon { range: r as i32, damage_bonus: power });
    }
    
    world.spawn(cb.build())
}

pub fn spawn_item(world: &mut World, x: u16, y: u16, raw: &RawItem) -> hecs::Entity {
    let mut cb = hecs::EntityBuilder::new();
    cb.add(Position { x, y });
    cb.add(Renderable { glyph: raw.glyph, fg: Color::Rgb(raw.color.0, raw.color.1, raw.color.2) });
    cb.add(RenderOrder::Item);
    cb.add(Item);
    cb.add(Name(raw.name.clone()));
    cb.add(ItemValue { price: raw.price });
    
    if let Some(h) = raw.potion { cb.add(Potion { heal_amount: h }); }
    if let Some(p) = raw.weapon { cb.add(Weapon { power_bonus: p }); }
    if let Some(d) = raw.armor { cb.add(Armor { defense_bonus: d }); }
    if let Some(r) = raw.ranged { cb.add(Ranged { range: r }); }
    if let Some((r, d)) = raw.ranged_weapon { cb.add(RangedWeapon { range: r, damage_bonus: d }); }
    if let Some(r) = raw.aoe { cb.add(AreaOfEffect { radius: r }); }
    if let Some(t) = raw.confusion { cb.add(Confusion { turns: t }); }
    if let Some((d, t)) = raw.poison { cb.add(Poison { damage: d, turns: t }); }
    if raw.ammo { cb.add(Ammunition); }
    if raw.consumable { cb.add(Consumable); }
    
    world.spawn(cb.build())
}

pub fn spawn_gold(world: &mut World, x: u16, y: u16, amount: i32) -> hecs::Entity {
    world.spawn((
        Position { x, y },
        Renderable { glyph: '*', fg: Color::Yellow },
        RenderOrder::Item,
        Name(format!("{} Gold", amount)),
        Gold { amount },
    ))
}

pub fn spawn_stairs(world: &mut World, x: u16, y: u16, down: bool) -> hecs::Entity {
    if down {
        world.spawn((
            Position { x, y },
            Renderable { glyph: '>', fg: Color::White },
            RenderOrder::Map,
            DownStairs,
            Name("Down Stairs".to_string()),
        ))
    } else {
        world.spawn((
            Position { x, y },
            Renderable { glyph: '<', fg: Color::White },
            RenderOrder::Map,
            UpStairs,
            Name("Up Stairs".to_string()),
        ))
    }
}

pub fn spawn_door(world: &mut World, x: u16, y: u16) -> hecs::Entity {
    world.spawn((
        Position { x, y },
        Renderable { glyph: '+', fg: Color::Indexed(94) },
        RenderOrder::Map,
        Door { open: false },
        Name("Door".to_string()),
    ))
}

pub fn spawn_trap(world: &mut World, x: u16, y: u16) -> hecs::Entity {
    world.spawn((
        Position { x, y },
        Renderable { glyph: '^', fg: Color::Red },
        RenderOrder::Trap,
        Trap { damage: 5, revealed: false },
        Name("Trap".to_string()),
    ))
}

pub fn spawn_merchant(world: &mut World, x: u16, y: u16) -> hecs::Entity {
    world.spawn((
        Position { x, y },
        Renderable { glyph: 'M', fg: Color::Rgb(255, 165, 0) },
        RenderOrder::Monster,
        Merchant,
        Name("Merchant".to_string()),
        CombatStats { max_hp: 100, hp: 100, defense: 10, power: 10 },
        Faction(FactionKind::Player),
        Viewshed { visible_tiles: 8 },
        AIPersonality(Personality::Tactical),
    ))
}

pub fn spawn_light_crystal(world: &mut World, x: u16, y: u16) -> hecs::Entity {
    world.spawn((
        Position { x, y },
        Renderable { glyph: '*', fg: Color::Rgb(100, 149, 237) },
        RenderOrder::Map,
        LightSource { range: 4, color: (100, 149, 237) },
        Name("Glowing Crystal".to_string()),
    ))
}

pub fn spawn_wisp(world: &mut World, x: u16, y: u16) -> hecs::Entity {
    world.spawn((
        Position { x, y },
        Renderable { glyph: '*', fg: Color::Cyan },
        RenderOrder::Map,
        LightSource { range: 4, color: (0, 255, 255) },
        Wisp,
        Name("Dungeon Wisp".to_string()),
    ))
}

pub fn spawn_item_in_backpack(world: &mut World, owner: hecs::Entity, raw: &RawItem) -> hecs::Entity {
    let mut cb = hecs::EntityBuilder::new();
    cb.add(Renderable { glyph: raw.glyph, fg: Color::Rgb(raw.color.0, raw.color.1, raw.color.2) });
    cb.add(RenderOrder::Item);
    cb.add(Item);
    cb.add(Name(raw.name.clone()));
    cb.add(ItemValue { price: raw.price });
    cb.add(InBackpack { owner });
    
    if let Some(h) = raw.potion { cb.add(Potion { heal_amount: h }); }
    if let Some(p) = raw.weapon { cb.add(Weapon { power_bonus: p }); }
    if let Some(d) = raw.armor { cb.add(Armor { defense_bonus: d }); }
    if let Some(r) = raw.ranged { cb.add(Ranged { range: r }); }
    if let Some((r, d)) = raw.ranged_weapon { cb.add(RangedWeapon { range: r, damage_bonus: d }); }
    if let Some(r) = raw.aoe { cb.add(AreaOfEffect { radius: r }); }
    if let Some(t) = raw.confusion { cb.add(Confusion { turns: t }); }
    if let Some((d, t)) = raw.poison { cb.add(Poison { damage: d, turns: t }); }
    if raw.ammo { cb.add(Ammunition); }
    if raw.consumable { cb.add(Consumable); }
    
    world.spawn(cb.build())
}
