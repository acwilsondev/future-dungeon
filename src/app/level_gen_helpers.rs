use crate::app::{App, Branch};
use crate::components::{Biome, FloorModifier};
use crate::map_builder::MapBuilder;
use ratatui::prelude::Color;

impl App {
    pub(crate) fn spawn_stairs(&mut self, mb: &MapBuilder) {
        crate::spawner::spawn_stairs(
            &mut self.world,
            mb.stairs_down.0,
            mb.stairs_down.1,
            true,
            (self.dungeon_level + 1, self.current_branch),
        );
        crate::spawner::spawn_stairs(
            &mut self.world,
            mb.stairs_up.0,
            mb.stairs_up.1,
            false,
            (
                self.dungeon_level.saturating_sub(1).max(1),
                self.current_branch,
            ),
        );

        if let Some(alt_down) = mb.stairs_down_alt {
            let alt_branch = if self.current_branch == Branch::Main {
                if self.dungeon_level.is_multiple_of(2) {
                    Branch::Gardens
                } else {
                    Branch::Vaults
                }
            } else {
                Branch::Main
            };
            crate::spawner::spawn_stairs(
                &mut self.world,
                alt_down.0,
                alt_down.1,
                true,
                (self.dungeon_level + 1, alt_branch),
            );
        }
    }

    pub(crate) fn spawn_ambient_features(&mut self, mb: &MapBuilder) {
        if mb.floor_modifier == FloorModifier::Dark {
            return;
        }

        let multiplier = if mb.floor_modifier == FloorModifier::Bright { 5 } else { 1 };
        let light_color = match mb.biome {
            Biome::Dungeon => Color::Rgb(100, 149, 237), // Cornflower Blue
            Biome::Caves => Color::Rgb(50, 205, 50),    // Lime Green
            Biome::Crypt => Color::Rgb(138, 43, 226),   // Blue Violet
            Biome::Temple => Color::Rgb(255, 69, 0),    // Orange Red
            Biome::Hell => Color::Rgb(255, 0, 0),       // Red
        };

        for (i, room) in mb.rooms.iter().enumerate().skip(1) {
            for _ in 0..multiplier {
                if i % 3 == 0 {
                    let center = room.center();
                    crate::spawner::spawn_light_crystal(
                        &mut self.world,
                        center.0 as u16,
                        center.1 as u16,
                        light_color,
                    );
                }
                if i % 5 == 0 {
                    let center = room.center();
                    crate::spawner::spawn_wisp(&mut self.world, center.0 as u16, center.1 as u16);
                }
            }
        }
    }
}
