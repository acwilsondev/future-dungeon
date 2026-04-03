use crate::app::App;
use crate::components::*;
use crate::map::TileType;
use bracket_pathfinding::prelude::*;

impl App {
    pub fn update_blocked_and_opaque(&mut self) {
        self.map.populate_blocked_and_opaque();

        for (_id, (pos, door)) in self.world.query::<(&Position, &Door)>().iter() {
            if !door.open {
                let idx = (pos.y as u16 * self.map.width + pos.x as u16) as usize;
                self.map.blocked[idx] = true;
                self.map.opaque[idx] = true;
            }
        }

        for (_id, (pos, _monster)) in self.world.query::<(&Position, &Monster)>().iter() {
            let idx = (pos.y as u16 * self.map.width + pos.x as u16) as usize;
            self.map.blocked[idx] = true;
        }

        for (_id, (pos, _merchant)) in self.world.query::<(&Position, &Merchant)>().iter() {
            let idx = (pos.y as u16 * self.map.width + pos.x as u16) as usize;
            self.map.blocked[idx] = true;
        }

        for (_id, (pos, _alchemy)) in self.world.query::<(&Position, &AlchemyStation)>().iter() {
            let idx = (pos.y as u16 * self.map.width + pos.x as u16) as usize;
            self.map.blocked[idx] = true;
        }
    }

    pub fn update_lighting(&mut self) {
        for l in self.map.light.iter_mut() { *l = 0.0; }

        let mut light_sources = Vec::new();
        for (_id, (pos, light)) in self.world.query::<(&Position, &LightSource)>().iter() {
            light_sources.push((*pos, *light));
        }

        for (pos, light) in light_sources {
            let idx_source = pos.y as usize * self.map.width as usize + pos.x as usize;
            self.map.light[idx_source] = (self.map.light[idx_source] + 1.0).min(1.5);

            let fov = field_of_view(Point::new(pos.x, pos.y), light.range, &self.map);
            for p in fov {
                if p.x >= 0 && p.x < self.map.width as i32 && p.y >= 0 && p.y < self.map.height as i32 {
                    let idx = p.y as usize * self.map.width as usize + p.x as usize;
                    let dist = (((p.x as f32 - pos.x as f32).powi(2) + (p.y as f32 - pos.y as f32).powi(2))).sqrt();
                    let intensity = 1.0 - (dist / light.range as f32);
                    self.map.light[idx] = (self.map.light[idx] + intensity).min(1.5); // Can be slightly over-bright
                }
            }
        }
    }

    pub fn update_sound(&mut self) {
        for s in self.map.sound.iter_mut() { *s = 0.0; }

        let mut noise_sources = Vec::new();
        for (_id, (pos, noise)) in self.world.query::<(&Position, &Noise)>().iter() {
            noise_sources.push((*pos, noise.amount));
        }

        for (pos, amount) in noise_sources {
            // Sound propagation using Dijkstra-like approach to "bend" around corners
            let mut dijkstra = DijkstraMap::new(self.map.width, self.map.height, &[], &self.map, 20.0);
            dijkstra.map[self.map.point2d_to_index(Point::new(pos.x, pos.y))] = 0.0;
            
            // Let's use a BFS for sound propagation
            let mut queue = std::collections::VecDeque::new();
            queue.push_back((pos.x, pos.y, amount));
            
            let start_idx = (pos.y as u16 * self.map.width + pos.x as u16) as usize;
            self.map.sound[start_idx] += amount;

            let mut visited = std::collections::HashSet::new();
            visited.insert((pos.x, pos.y));

            while let Some((cx, cy, current_amount)) = queue.pop_front() {
                if current_amount <= 0.1 { continue; }

                for (dx, dy) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let nx = cx as i16 + dx;
                    let ny = cy as i16 + dy;
                    if nx >= 0 && nx < self.map.width as i16 && ny >= 0 && ny < self.map.height as i16 {
                        let nux = nx as u16;
                        let nuy = ny as u16;
                        if visited.contains(&(nux, nuy)) { continue; }
                        
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
            let (id, (pos, _)) = player_query.iter().next().expect("Player not found");
            let range = self.world.get::<&Viewshed>(id).map(|v| v.visible_tiles).unwrap_or(8);
            (*pos, range)
        };

        // Calculate broad LOS (max 20 tiles)
        let fov = field_of_view(Point::new(pos.x, pos.y), 25, &self.map);
        for v in &mut self.map.visible { *v = false; }
        for p in fov {
            if p.x >= 0 && p.x < self.map.width as i32 && p.y >= 0 && p.y < self.map.height as i32 {
                let idx = (p.y as u16 * self.map.width + p.x as u16) as usize;
                let dist = (((p.x as f32 - pos.x as f32).powi(2) + (p.y as f32 - pos.y as f32).powi(2))).sqrt();

                // Visible if within player's sight range AND the tile is lit
                if dist <= range as f32 && self.map.light[idx] > 0.1 {
                    self.map.visible[idx] = true;
                    self.map.revealed[idx] = true;
                }
            }
        }

        // Record encountered monsters
        for (_id, (pos, name, _)) in self.world.query::<(&Position, &Name, &Monster)>().iter() {
            let idx = (pos.y as u16 * self.map.width + pos.x as u16) as usize;
            if self.map.visible[idx] {
                self.encountered_monsters.insert(name.0.clone());
            }
        }
    }

    pub fn generate_noise(&mut self, x: u16, y: u16, amount: f32) {
        self.world.spawn((Position { x, y }, Noise { amount }));
    }
}
