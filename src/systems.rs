use hecs::World;
use crate::components::*;
use crate::app::{RunState, VisualEffect};
use ratatui::prelude::Color;

pub fn move_player(world: &mut World, dx: i16, dy: i16, log: &mut Vec<String>, effects: &mut Vec<VisualEffect>, state: &mut RunState, shop_cursor: &mut usize, active_merchant: &mut Option<hecs::Entity>) {
    let (new_x, new_y, player_power) = {
        let mut player_query = world.query::<(&Position, &Player, &CombatStats)>();
        let Some((_, (pos, _, player_stats))) = player_query.iter().next() else { return; };
        ((pos.x as i16 + dx).max(0) as u16, (pos.y as i16 + dy).max(0) as u16, player_stats.power)
    };

    let mut target_interactable = None;
    for (id, (pos, _)) in world.query::<(&Position, &Monster)>().iter() {
        if pos.x == new_x && pos.y == new_y { target_interactable = Some(id); break; }
    }
    if target_interactable.is_none() {
        for (id, (pos, _)) in world.query::<(&Position, &Merchant)>().iter() {
            if pos.x == new_x && pos.y == new_y { target_interactable = Some(id); break; }
        }
    }

    if let Some(target_id) = target_interactable {
        // Check if it's a Merchant
        if world.get::<&Merchant>(target_id).is_ok() {
            *active_merchant = Some(target_id);
            *state = RunState::ShowShop;
            *shop_cursor = 0;
            log.push("You talk to the Merchant.".to_string());
            return;
        }

        let mut dead = false;
        let monster_name = world.get::<&Name>(target_id).map(|n| n.0.clone()).unwrap_or("Something".to_string());
        let mut xp_reward = 0;
        if let Ok(mut monster_stats) = world.get::<&mut CombatStats>(target_id) {
            let damage = (player_power - monster_stats.defense).max(0);
            monster_stats.hp -= damage;
            log.push(format!("You hit {} for {} damage!", monster_name, damage));
            effects.push(VisualEffect::Flash { x: new_x, y: new_y, glyph: '*', fg: Color::Red, bg: None, duration: 5 });
            if monster_stats.hp <= 0 {
                dead = true;
                if let Ok(exp) = world.get::<&Experience>(target_id) {
                    xp_reward = exp.xp_reward;
                }
            }
        }
        if !dead {
            let _ = world.insert_one(target_id, LastHitByPlayer);
        }
        if dead {
            log.push(format!("{} dies!", monster_name));
            if let Err(e) = world.despawn(target_id) {
                log::error!("Failed to despawn monster {:?}: {}", target_id, e);
            }
            // XP handling would need to be passed in or handled by App
            // For now, let's assume App handles XP after this returns or we pass a callback
        }
        if *state != RunState::LevelUp {
            *state = RunState::MonsterTurn;
        }
        return;
    }

    // Check for blocking
    // Need Map access for this... 
}
