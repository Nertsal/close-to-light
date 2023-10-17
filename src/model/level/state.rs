use super::*;

/// A renderable state of the level at some given time.
#[derive(Debug)]
pub struct LevelState {
    /// The time at which the render has been done.
    beat_time: Time,
    /// The time after which events are not rendered.
    ignore_after: Option<Time>,
    pub config: LevelConfig,
    pub lights: Vec<Light>,
    pub telegraphs: Vec<LightTelegraph>,
    pub is_finished: bool,
}

impl Default for LevelState {
    fn default() -> Self {
        Self {
            beat_time: Time::ZERO,
            ignore_after: None,
            config: LevelConfig::default(),
            lights: Vec::new(),
            telegraphs: Vec::new(),
            is_finished: false,
        }
    }
}

impl LevelState {
    pub fn render(level: &Level, beat_time: Time, ignore_after: Option<Time>) -> Self {
        let mut state = Self {
            beat_time,
            ignore_after,
            lights: Vec::new(),
            telegraphs: Vec::new(),
            is_finished: true,
            config: level.config.clone(),
        };

        for (i, e) in level.events.iter().enumerate() {
            state.render_event(e, Some(i));
        }
        state
    }

    pub fn render_event(&mut self, event: &TimedEvent, event_id: Option<usize>) {
        if self.beat_time < event.beat {
            self.is_finished = false;
            return;
        }

        if let Some(time) = self.ignore_after {
            if event.beat > time {
                return;
            }
        }

        let time = self.beat_time - event.beat;

        match &event.event {
            Event::Theme(new_theme) => self.config.theme = new_theme.clone(),
            Event::Light(event) => {
                let light = event.light.clone().instantiate(event_id);
                let mut tele = light.clone().into_telegraph(event.telegraph.clone());
                let duration = event.light.movement.duration();

                // Telegraph
                if time < duration {
                    let transform = event.light.movement.get(time);
                    tele.light.collider = light.collider.transformed(transform);
                    self.telegraphs.push(tele.clone());
                }

                // Light
                let time = time - event.telegraph.precede_time;
                if time > Time::ZERO && time < duration {
                    let transform = event.light.movement.get(time);
                    tele.light.collider = light.collider.transformed(transform);
                    self.lights.push(tele.light.clone());
                }
            }
        }

        self.is_finished = self.is_finished && self.lights.is_empty() && self.telegraphs.is_empty();
    }
}
