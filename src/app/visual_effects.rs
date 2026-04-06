use crate::app::{App, VisualEffect};

impl App {
    pub fn on_tick(&mut self) {
        let mut still_active = Vec::new();

        for effect in self.effects.drain(..) {
            match effect {
                VisualEffect::Flash {
                    x,
                    y,
                    glyph,
                    fg,
                    bg,
                    duration,
                } => {
                    if duration > 1 {
                        still_active.push(VisualEffect::Flash {
                            x,
                            y,
                            glyph,
                            fg,
                            bg,
                            duration: duration - 1,
                        });
                    }
                }
                VisualEffect::Projectile {
                    path,
                    glyph,
                    fg,
                    frame,
                    speed,
                } => {
                    let new_frame = frame + 1;
                    if new_frame < (path.len() as u32 * speed) {
                        still_active.push(VisualEffect::Projectile {
                            path,
                            glyph,
                            fg,
                            frame: new_frame,
                            speed,
                        });
                    }
                }
            }
        }
        self.effects = still_active;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::prelude::Color;

    #[test]
    fn test_flash_effect_progression() {
        let mut app = App::new_random();
        app.effects.push(VisualEffect::Flash {
            x: 0, y: 0, glyph: '*', fg: Color::Red, bg: None, duration: 2
        });

        app.on_tick();
        assert_eq!(app.effects.len(), 1);
        if let VisualEffect::Flash { duration, .. } = app.effects[0] {
            assert_eq!(duration, 1);
        }

        app.on_tick();
        assert_eq!(app.effects.len(), 0);
    }

    #[test]
    fn test_projectile_effect_progression() {
        let mut app = App::new_random();
        app.effects.push(VisualEffect::Projectile {
            path: vec![(0,0), (1,1)],
            glyph: '/',
            fg: Color::White,
            frame: 0,
            speed: 1,
        });

        app.on_tick();
        assert_eq!(app.effects.len(), 1);
        if let VisualEffect::Projectile { frame, .. } = app.effects[0] {
            assert_eq!(frame, 1);
        }

        app.on_tick();
        assert_eq!(app.effects.len(), 0); // frame 2 >= 2*1
    }
}
