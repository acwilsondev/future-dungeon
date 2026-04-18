use crate::app::{App, RunState, Star, VisualEffect};
use rand::Rng;

impl App {
    pub fn init_stars(&mut self) {
        for _ in 0..100 {
            self.stars.push(Star {
                x: self.rng.random_range(0.0..200.0),
                y: self.rng.random_range(0.0..100.0),
                speed: self.rng.random_range(0.05..0.2),
                brightness: self.rng.random_range(50..255) as u8,
            });
        }
    }

    pub fn on_tick(&mut self) {
        // Update Stars
        if self.state == RunState::MainMenu {
            for star in self.stars.iter_mut() {
                star.x += star.speed;
                if star.x > 200.0 {
                    star.x = 0.0;
                    star.y = self.rng.random_range(0.0..100.0);
                }
            }
        }

        // Update Visual Effects
        self.effects.retain_mut(|effect| {
            match effect {
                VisualEffect::Flash { duration, .. } => {
                    if *duration > 0 {
                        *duration -= 1;
                        true
                    } else {
                        false
                    }
                }
                VisualEffect::Projectile { frame, speed, path, .. } => {
                    *frame += 1;
                    (*frame / *speed) < path.len() as u32
                }
            }
        });
    }
}
