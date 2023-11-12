use super::*;

/// A renderable state of the level at some given time.
#[derive(Debug)]
pub struct LevelState {
    /// The time at which the render has been done.
    beat_time: Time,
    /// The time after which events are not rendered.
    ignore_after: Option<Time>,
    /// Whether the palette should be swapped.
    pub swap_palette: bool,
    pub lights: Vec<Light>,
    pub telegraphs: Vec<LightTelegraph>,
    pub is_finished: bool,
}

impl Default for LevelState {
    fn default() -> Self {
        Self {
            beat_time: Time::ZERO,
            ignore_after: None,
            swap_palette: false,
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
            swap_palette: false,
            lights: Vec::new(),
            telegraphs: Vec::new(),
            is_finished: true,
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
            Event::PaletteSwap => self.swap_palette = !self.swap_palette,
            Event::Light(event) => {
                let (telegraph, light) = render_light(event, time, event_id);
                self.telegraphs.extend(telegraph);
                self.lights.extend(light);
            }
        }

        self.is_finished = self.is_finished && self.lights.is_empty() && self.telegraphs.is_empty();
    }
}

pub fn render_light(
    event: &LightEvent,
    relative_time: Time,
    event_id: Option<usize>,
) -> (Option<LightTelegraph>, Option<Light>) {
    let light = event.light.clone().instantiate(event_id);
    let mut tele = light.clone().into_telegraph(event.telegraph.clone());
    let duration = event.light.movement.total_duration();

    // Telegraph
    let telegraph = (relative_time < duration).then(|| {
        let transform = event.light.movement.get(relative_time);
        tele.light.collider = light.collider.transformed(transform);
        tele.clone()
    });

    // Light
    let time = relative_time - event.telegraph.precede_time;
    let light = (time > Time::ZERO && time < duration).then(|| {
        let transform = event.light.movement.get(time);
        tele.light.collider = light.collider.transformed(transform);
        tele.light
    });

    (telegraph, light)
}
