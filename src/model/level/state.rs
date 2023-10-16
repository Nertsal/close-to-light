use super::*;

/// A renderable state of the level at some given time.
#[derive(Debug, Default)]
pub struct LevelState {
    pub lights: Vec<Light>,
    pub telegraphs: Vec<LightTelegraph>,
}

impl LevelState {
    pub fn render(level: &Level, beat_time: Time) -> Self {
        let mut lights = Vec::new();
        let mut telegraphs = Vec::new();

        let time = beat_time * level.beat_time();

        let mut render_light = |event: &TimedEvent| {
            if event.beat <= beat_time {
                let start = event.beat * level.beat_time();
                let time = time - start;

                match &event.event {
                    Event::Light(event) => {
                        let light = event.light.clone().instantiate(level.beat_time());
                        let mut tele =
                            light.into_telegraph(event.telegraph.clone(), level.beat_time());
                        let duration = tele.light.movement.duration();

                        // Telegraph
                        if time < duration {
                            let transform = tele.light.movement.get(time);
                            tele.light.collider = tele.light.base_collider.transformed(transform);
                            telegraphs.push(tele.clone());
                        }

                        // Light
                        let time = time - tele.spawn_timer;
                        if time > Time::ZERO && time < duration {
                            let transform = tele.light.movement.get(time);
                            tele.light.collider = tele.light.base_collider.transformed(transform);
                            lights.push(tele.light.clone());
                        }
                    }
                }
            }
        };

        for e in &level.events {
            render_light(e);
        }

        Self { lights, telegraphs }
    }
}