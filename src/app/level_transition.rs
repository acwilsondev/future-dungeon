use crate::app::{App, RunState, EntitySnapshot, LevelData};
use crate::components::*;

impl App {
    pub fn go_to_level(&mut self, destination: (u16, Branch)) {
        let from = (self.dungeon_level, self.current_branch);
        
        if self.dungeon_level <= 1 && destination.0 < 1 {
            if self.escaping {
                self.state = RunState::Victory;
                self.log.push("You escape the dungeon with the Amulet! You win!".to_string());
            } else {
                self.log.push("You cannot go further up without the Amulet!".to_string());
            }
            return;
        }

        self.pack_entities();
        let current_entities = self.entities.clone();
        let player_snapshot = current_entities.iter().find(|e| e.is_player).cloned().expect("Player entity not found during level transition");
        let level_entities: Vec<EntitySnapshot> = current_entities.into_iter().filter(|e| !e.is_player).collect();

        self.levels.insert((self.dungeon_level, self.current_branch), LevelData {
            map: self.map.clone(),
            entities: level_entities,
        });

        let going_down = destination.0 > self.dungeon_level;
        
        self.dungeon_level = destination.0;
        self.current_branch = destination.1;

        if let Some(level_data) = self.levels.get(&(self.dungeon_level, self.current_branch)) {
            self.map = level_data.map.clone();
            self.entities = level_data.entities.clone();
            self.entities.push(player_snapshot);
            self.unpack_entities();
            let mut stairs_pos = (0, 0);
            
            if going_down {
                for (_, (pos, stairs)) in self.world.query::<(&Position, &UpStairs)>().iter() {
                    if stairs.destination == from {
                        stairs_pos = (pos.x, pos.y);
                        break;
                    }
                    stairs_pos = (pos.x, pos.y); // Fallback
                }
            } else {
                for (_, (pos, stairs)) in self.world.query::<(&Position, &DownStairs)>().iter() {
                    if stairs.destination == from {
                        stairs_pos = (pos.x, pos.y);
                        break;
                    }
                    stairs_pos = (pos.x, pos.y); // Fallback
                }
            }

            let mut player_query = self.world.query::<(&mut Position, &Player)>();
            if let Some((_, (pos, _))) = player_query.iter().next() {
                pos.x = stairs_pos.0;
                pos.y = stairs_pos.1;
            }
        } else {
            self.generate_level(Some(player_snapshot));
        }

        let branch_name = match self.current_branch {
            Branch::Main => "Main Dungeon",
            Branch::Gardens => "Overgrown Gardens",
            Branch::Vaults => "Frozen Vaults",
        };

        if going_down {
            self.log.push(format!("You descend to level {} of {}.", self.dungeon_level, branch_name));
        } else {
            self.log.push(format!("You ascend to level {} of {}.", self.dungeon_level, branch_name));
        }
    }

    pub fn try_level_transition(&mut self) {
        let player_pos = {
            let mut player_query = self.world.query::<(&Position, &Player)>();
            let (_, (pos, _)) = player_query.iter().next().expect("Player not found");
            *pos
        };
        let mut transition_down = None;
        let mut transition_up = None;

        for (_, (pos, stairs)) in self.world.query::<(&Position, &DownStairs)>().iter() {
            if pos.x == player_pos.x && pos.y == player_pos.y {
                transition_down = Some(stairs.destination);
            }
        }
        for (_, (pos, stairs)) in self.world.query::<(&Position, &UpStairs)>().iter() {
            if pos.x == player_pos.x && pos.y == player_pos.y {
                transition_up = Some(stairs.destination);
            }
        }

        if let Some(dest) = transition_down {
            self.go_to_level(dest);
        } else if let Some(dest) = transition_up {
            self.go_to_level(dest);
        } else {
            self.log.push("There are no stairs here.".to_string());
        }
    }
}
