use crate::app::App;
use crate::components::*;
use crate::map::TileType;
use bracket_pathfinding::prelude::*;

impl App {
    pub fn update_blocked_and_opaque(&mut self) {
        self.map.populate_blocked_and_opaque();

        for (_id, (pos, door)) in self.world.query::<(&Position, &Door)>().iter() {
            if !door.open {
                let idx = (pos.y * self.map.width + pos.x) as usize;
                self.map.blocked[idx] = true;
                self.map.opaque[idx] = true;
            }
        }

        for (_id, (pos, _monster)) in self.world.query::<(&Position, &Monster)>().iter() {
            let idx = (pos.y * self.map.width + pos.x) as usize;
            self.map.blocked[idx] = true;
        }

        for (_id, (pos, _merchant)) in self.world.query::<(&Position, &Merchant)>().iter() {
            let idx = (pos.y * self.map.width + pos.x) as usize;
            self.map.blocked[idx] = true;
        }

        for (_id, (pos, _alchemy)) in self.world.query::<(&Position, &AlchemyStation)>().iter() {
            let idx = (pos.y * self.map.width + pos.x) as usize;
            self.map.blocked[idx] = true;
        }

        for (_id, (pos, _altar)) in self.world.query::<(&Position, &HolyAltar)>().iter() {
            let idx = (pos.y * self.map.width + pos.x) as usize;
            self.map.blocked[idx] = true;
        }

        for (_id, (pos, _shrine)) in self.world.query::<(&Position, &ResetShrine)>().iter() {
            let idx = (pos.y * self.map.width + pos.x) as usize;
            self.map.blocked[idx] = true;
        }
    }

    pub fn update_lighting(&mut self) {
        for l in self.map.light.iter_mut() {
            *l = 0.0;
        }

        let mut light_sources = Vec::new();
        for (_id, (pos, light)) in self.world.query::<(&Position, &LightSource)>().iter() {
            light_sources.push((*pos, *light));
        }

        for (pos, light) in light_sources {
            let idx_source = pos.y as usize * self.map.width as usize + pos.x as usize;
            self.map.light[idx_source] = (self.map.light[idx_source] + 1.0).min(1.5);

            let fov = field_of_view(Point::new(pos.x, pos.y), light.range, &self.map);
            for p in fov {
                if p.x >= 0
                    && p.x < self.map.width as i32
                    && p.y >= 0
                    && p.y < self.map.height as i32
                {
                    let idx = p.y as usize * self.map.width as usize + p.x as usize;
                    let dist = ((p.x as f32 - pos.x as f32).powi(2)
                        + (p.y as f32 - pos.y as f32).powi(2))
                    .sqrt();
                    let intensity = 1.0 - (dist / light.range as f32);
                    self.map.light[idx] = (self.map.light[idx] + intensity).min(1.5);
                    // Can be slightly over-bright
                }
            }
        }
    }

    pub fn update_sound(&mut self) {
        for s in self.map.sound.iter_mut() {
            *s = 0.0;
        }

        let mut noise_sources = Vec::new();
        for (_id, (pos, noise)) in self.world.query::<(&Position, &Noise)>().iter() {
            noise_sources.push((*pos, noise.amount));
        }

        for (pos, amount) in noise_sources {
            // Sound propagation using Dijkstra-like approach to "bend" around corners
            let mut dijkstra =
                DijkstraMap::new(self.map.width, self.map.height, &[], &self.map, 20.0);
            dijkstra.map[self.map.point2d_to_index(Point::new(pos.x, pos.y))] = 0.0;

            // Let's use a BFS for sound propagation
            let mut queue = std::collections::VecDeque::new();
            queue.push_back((pos.x, pos.y, amount));

            let start_idx = (pos.y * self.map.width + pos.x) as usize;
            self.map.sound[start_idx] += amount;

            let mut visited = std::collections::HashSet::new();
            visited.insert((pos.x, pos.y));

            while let Some((cx, cy, current_amount)) = queue.pop_front() {
                if current_amount <= 0.1 {
                    continue;
                }

                for (dx, dy) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let nx = cx as i16 + dx;
                    let ny = cy as i16 + dy;
                    if nx >= 0
                        && nx < self.map.width as i16
                        && ny >= 0
                        && ny < self.map.height as i16
                    {
                        let nux = nx as u16;
                        let nuy = ny as u16;
                        if visited.contains(&(nux, nuy)) {
                            continue;
                        }

                        let idx = (nuy * self.map.width + nux) as usize;
                        let attenuation = if self.map.tiles[idx] == TileType::Wall {
                            4.0 // Walls muffle sound significantly
                        } else {
                            1.1 // Open air attenuation
                        };

                        let next_amount = current_amount - attenuation;
                        if next_amount > 0.0 {
                            self.map.sound[idx] += next_amount;
                            visited.insert((nux, nuy));
                            queue.push_back((nux, nuy, next_amount));
                        }
                    }
                }
            }
        }
    }

    pub fn update_fov(&mut self) {
        self.update_lighting();
        self.update_sound();
        let (pos, range) = {
            let mut player_query = self.world.query::<(&Position, &Player)>();
            if let Some((id, (pos, _))) = player_query.iter().next() {
                let range = self
                    .world
                    .get::<&Viewshed>(id)
                    .map(|v| v.visible_tiles)
                    .unwrap_or(8);
                (*pos, range)
            } else {
                return;
            }
        };

        // Calculate broad LOS (max 20 tiles)
        let fov = field_of_view(Point::new(pos.x, pos.y), 25, &self.map);
        for v in &mut self.map.visible {
            *v = false;
        }
        for p in fov {
            if p.x >= 0 && p.x < self.map.width as i32 && p.y >= 0 && p.y < self.map.height as i32 {
                let idx = (p.y as u16 * self.map.width + p.x as u16) as usize;
                let dist = ((p.x as f32 - pos.x as f32).powi(2)
                    + (p.y as f32 - pos.y as f32).powi(2))
                .sqrt();

                // Visible if within player's sight range AND the tile is lit
                if dist <= range as f32 && self.map.light[idx] > 0.1 {
                    self.map.visible[idx] = true;
                    self.map.revealed[idx] = true;
                }
            }
        }

        // Record encountered monsters
        for (_id, (pos, name, _)) in self.world.query::<(&Position, &Name, &Monster)>().iter() {
            let idx = (pos.y * self.map.width + pos.x) as usize;
            if self.map.visible[idx] {
                self.encountered_monsters.insert(name.0.clone());
            }
        }
    }

    pub fn generate_noise(&mut self, x: u16, y: u16, amount: f32) {
        self.world.spawn((Position { x, y }, Noise { amount }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hecs::World;

    fn setup_test_app() -> App {
        let mut app = App::new_test(42);
        app.world = World::new();
        app.map = crate::map::Map::new(80, 50);
        for t in app.map.tiles.iter_mut() {
            *t = TileType::Floor;
        }
        app.update_blocked_and_opaque();
        app
    }

    #[test]
    fn test_blocking_entities() {
        let mut app = setup_test_app();
        app.world.spawn((Position { x: 10, y: 10 }, Monster, Name("M".to_string())));
        app.world.spawn((Position { x: 11, y: 10 }, Merchant));
        app.world.spawn((Position { x: 12, y: 10 }, AlchemyStation));
        app.world.spawn((Position { x: 13, y: 10 }, Door { open: false }));
        app.world.spawn((Position { x: 14, y: 10 }, Door { open: true }));

        app.update_blocked_and_opaque();

        assert!(app.map.blocked[(10 * 80 + 10) as usize]);
        assert!(app.map.blocked[(10 * 80 + 11) as usize]);
        assert!(app.map.blocked[(10 * 80 + 12) as usize]);
        assert!(app.map.blocked[(10 * 80 + 13) as usize]);
        assert!(!app.map.blocked[(10 * 80 + 14) as usize]);
    }

    #[test]
    fn test_lighting_propagation() {
        let mut app = setup_test_app();
        app.world.spawn((
            Position { x: 10, y: 10 },
            LightSource {
                range: 5,
                base_range: 5,
                color: (255, 255, 255),
                remaining_turns: None,
                flicker: false,
            }
        ));

        app.update_lighting();

        assert!(app.map.light[(10 * 80 + 10) as usize] > 1.0);
        assert!(app.map.light[(10 * 80 + 12) as usize] > 0.5);
        assert!(app.map.light[(10 * 80 + 16) as usize] < 0.1);
    }

    #[test]
    fn test_sound_propagation() {
        let mut app = setup_test_app();
        // Place a wall between 10,10 and 12,10
        app.map.tiles[(10 * 80 + 11) as usize] = TileType::Wall;
        app.generate_noise(10, 10, 10.0);

        app.update_sound();

        let loud_idx = (10 * 80 + 10) as usize;
        let muffled_idx = (10 * 80 + 12) as usize;
        assert!(app.map.sound[loud_idx] > 9.0);
        assert!(app.map.sound[muffled_idx] < 5.0); // Muffled by wall
    }

    #[test]
    fn test_fov_with_light() {
        let mut app = setup_test_app();
        let _player = app.world.spawn((
            Player,
            Position { x: 10, y: 10 },
            Viewshed { visible_tiles: 10 }
        ));

        // Case 1: Player at 10,10, target at 12,10, but NO LIGHT
        app.update_fov();
        assert!(!app.map.visible[(10 * 80 + 12) as usize]);

        // Case 2: Target is LIT by a light source
        app.world.spawn((
            Position { x: 12, y: 10 },
            LightSource {
                range: 5,
                base_range: 5,
                color: (255, 255, 255),
                remaining_turns: None,
                flicker: false,
            }
        ));
        app.update_fov();
        assert!(app.map.visible[(10 * 80 + 12) as usize]);
    }
}
