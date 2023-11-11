use super::*;

/// A renderable state of the level at some given time.
#[derive(Debug)]
pub struct LevelState {
    /// The time at which the render has been done.
    beat_time: Time,
    /// The time after which events are not rendered.
    ignore_after: Option<Time>,
    pub config: LevelConfig,
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
            config: LevelConfig::default(),
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
) -> (Vec<LightTelegraph>, Option<Light>) {
    let movement = &event.light.movement;
    let base_light = event.light.clone().instantiate(event_id);
    let base_tele = base_light.clone().into_telegraph(event.telegraph.clone());
    let duration = event.light.movement.total_duration();

    // Telegraph
    let telegraphs = if relative_time > duration {
        vec![]
    } else {
        let transform = event.light.movement.get(relative_time);
        let mut main_tele = base_tele.clone();
        main_tele.light.collider = base_light.collider.transformed(transform);

        // TODO: config
        let sustain_time = r32(1.0);
        let fade_time = r32(0.5);
        let sustain_scale = r32(0.5);

        let mut last_pos = movement.initial.translation;
        let waypoints = movement
            .timed_positions()
            .take(movement.key_frames.len()) // Ignore the last position
            .skip(1) // Ignore the initial position
            .filter_map(|(_, mut transform, time)| {
                let relative_time = relative_time - time;
                let waypoint = (last_pos != transform.translation
                    && (Time::ZERO..sustain_time + fade_time).contains(&relative_time))
                .then(|| {
                    let scale = if relative_time < sustain_time {
                        let t = relative_time / sustain_time;
                        sustain_scale
                            + crate::util::smoothstep(Time::ONE - t) * (Coord::ONE - sustain_scale)
                    } else {
                        let t = (relative_time - sustain_time) / fade_time;
                        sustain_scale * crate::util::smoothstep(Time::ONE - t)
                    };
                    transform.scale *= scale;
                    let mut tele = base_tele.clone();
                    tele.light.collider = base_light.collider.transformed(transform);
                    tele
                });
                last_pos = transform.translation;
                waypoint
            });

        std::iter::once(main_tele).chain(waypoints).collect()
    };

    // Light
    let relative_time = relative_time - event.telegraph.precede_time;
    let light = (relative_time > Time::ZERO && relative_time < duration).then(|| {
        let transform = event.light.movement.get(relative_time);
        let mut main_light = base_tele.light;
        main_light.collider = base_light.collider.transformed(transform);
        main_light
    });

    (telegraphs, light)
}
