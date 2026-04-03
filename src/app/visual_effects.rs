use crate::app::{App, VisualEffect};

impl App {
    pub fn on_tick(&mut self) {
        let mut still_active = Vec::new();

        for effect in self.effects.drain(..) {
            match effect {
                VisualEffect::Flash { x, y, glyph, fg, bg, duration } => {
                    if duration > 1 {
                        still_active.push(VisualEffect::Flash { x, y, glyph, fg, bg, duration: duration - 1 });
                    }
                }
                VisualEffect::Projectile { path, glyph, fg, frame, speed } => {
                    let new_frame = frame + 1;
                    if new_frame < (path.len() as u32 * speed) {
                        still_active.push(VisualEffect::Projectile { path, glyph, fg, frame: new_frame, speed });
                    }
                }
            }
        }
        self.effects = still_active;
    }
}
