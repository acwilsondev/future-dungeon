use crate::app::{App, EntitySnapshot, Branch};
use crate::components::*;
use hecs::World;

impl App {
    pub fn pack_entities(&mut self) {
        self.entities.clear();
        for (id, (render, render_order)) in self.world.query::<(&Renderable, &RenderOrder)>().iter() {
            let pos = self.world.get::<&Position>(id).ok().map(|p| *p);
            let name = self.world.get::<&Name>(id).ok().map(|n| (*n).clone());
            let stats = self.world.get::<&CombatStats>(id).ok().map(|s| *s);
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
            let obfuscated_name = self.world.get::<&ObfuscatedName>(id).ok().map(|n| (*n).clone());
            let cursed = self.world.get::<&Cursed>(id).ok().map(|c| *c);
            let equippable = self.world.get::<&Equippable>(id).ok().map(|e| *e);
            let equipped = self.world.get::<&Equipped>(id).ok().map(|e| (*e).clone());
            
            self.entities.push(EntitySnapshot {
                pos, render: *render, render_order: *render_order, name, stats, potion, weapon, armor, door, trap, ranged, 
                ranged_weapon, aoe, confusion, poison, strength, speed,
                faction, viewshed, personality, experience, perks, alert_state, hearing, boss, light_source, gold, item_value,
                obfuscated_name, cursed, equippable, equipped,
                last_hit_by_player: self.world.get::<&LastHitByPlayer>(id).is_ok(),
                is_merchant: self.world.get::<&Merchant>(id).is_ok(),
                ammo: self.world.get::<&Ammunition>(id).is_ok(),
                consumable: self.world.get::<&Consumable>(id).is_ok(),
                in_backpack: self.world.get::<&InBackpack>(id).is_ok(),
                is_player: self.world.get::<&Player>(id).is_ok(),
                is_monster: self.world.get::<&Monster>(id).is_ok(),
                is_wisp: self.world.get::<&Wisp>(id).is_ok(),
                is_item: self.world.get::<&Item>(id).is_ok(),
                is_down_stairs: self.world.get::<&DownStairs>(id).is_ok(),
                is_up_stairs: self.world.get::<&UpStairs>(id).is_ok(),
                destination: self.world.get::<&DownStairs>(id).ok().map(|s| s.destination)
                    .or_else(|| self.world.get::<&UpStairs>(id).ok().map(|s| s.destination)),
            });
        }
    }

    pub fn unpack_entities(&mut self) {
        self.world = World::new();
        let mut player_entity = None;
        let mut in_backpack_markers = Vec::new();

        for e in &self.entities {
            let mut cb = hecs::EntityBuilder::new();
            if let Some(pos) = e.pos { cb.add(pos); }
            cb.add(e.render);
            cb.add(e.render_order);
            if let Some(ref name) = e.name { cb.add(name.clone()); }
            if let Some(stats) = e.stats { cb.add(stats); }
            if let Some(potion) = e.potion { cb.add(potion); }
            if let Some(weapon) = e.weapon { cb.add(weapon); }
            if let Some(armor) = e.armor { cb.add(armor); }
            if let Some(door) = e.door { cb.add(door); }
            if let Some(trap) = e.trap { cb.add(trap); }
            if let Some(ranged) = e.ranged { cb.add(ranged); }
            if let Some(ranged_weapon) = e.ranged_weapon { cb.add(ranged_weapon); }
            if let Some(aoe) = e.aoe { cb.add(aoe); }
            if let Some(confusion) = e.confusion { cb.add(confusion); }
            if let Some(poison) = e.poison { cb.add(poison); }
            if let Some(strength) = e.strength { cb.add(strength); }
            if let Some(speed) = e.speed { cb.add(speed); }
            if let Some(faction) = e.faction { cb.add(faction); }
            if let Some(viewshed) = e.viewshed { cb.add(viewshed); }
            if let Some(personality) = e.personality { cb.add(personality); }
            if let Some(experience) = e.experience { cb.add(experience); }
            if let Some(perks) = e.perks.clone() { cb.add(perks); }
            if let Some(alert_state) = e.alert_state { cb.add(alert_state); }
            if let Some(hearing) = e.hearing { cb.add(hearing); }
            if let Some(boss) = e.boss.clone() { cb.add(boss); }
            if let Some(light_source) = e.light_source { cb.add(light_source); }
            if let Some(gold) = e.gold { cb.add(gold); }
            if let Some(item_value) = e.item_value { cb.add(item_value); }
            if let Some(obfuscated_name) = e.obfuscated_name.clone() { cb.add(obfuscated_name); }
            if let Some(cursed) = e.cursed { cb.add(cursed); }
            if let Some(equippable) = e.equippable { cb.add(equippable); }
            if let Some(equipped) = e.equipped.clone() { cb.add(equipped); }
            if e.last_hit_by_player { cb.add(LastHitByPlayer); }
            if e.is_merchant { cb.add(Merchant); }
            if e.ammo { cb.add(Ammunition); }
            if e.consumable { cb.add(Consumable); }
            if e.is_player { cb.add(Player); }
            if e.is_monster { cb.add(Monster); }
            if e.is_wisp { cb.add(Wisp); }
            if e.is_item { cb.add(Item); }
            if e.is_down_stairs { cb.add(DownStairs { destination: e.destination.unwrap_or((0, Branch::Main)) }); }
            if e.is_up_stairs { cb.add(UpStairs { destination: e.destination.unwrap_or((0, Branch::Main)) }); }
            let entity = self.world.spawn(cb.build());
            if e.is_player { player_entity = Some(entity); }
            if e.in_backpack { in_backpack_markers.push(entity); }
        }

        if let Some(player) = player_entity {
            for id in in_backpack_markers {
                self.world.insert_one(id, InBackpack { owner: player }).expect("Failed to insert InBackpack component during unpack");
            }
        }

        self.map.visible = vec![false; (self.map.width * self.map.height) as usize];
        self.update_blocked_and_opaque();
        self.update_fov();
    }
}
