use super::*;

/// A renderable state of the level at some given time.
#[derive(Debug)]
pub struct LevelState {
    /// The time at which the render has been done.
    time: Time,
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
            time: Time::ZERO,
            ignore_after: None,
            swap_palette: false,
            lights: Vec::new(),
            telegraphs: Vec::new(),
            is_finished: false,
        }
    }
}

impl LevelState {
    pub fn render(
        level: &Level,
        config: &LevelConfig,
        time: Time,
        ignore_after: Option<Time>,
    ) -> Self {
        let mut state = Self {
            time,
            ignore_after,
            swap_palette: false,
            lights: Vec::new(),
            telegraphs: Vec::new(),
            is_finished: true,
        };

        for (i, e) in level.events.iter().enumerate() {
            state.render_event(e, Some(i), config);
        }
        state
    }

    pub fn render_event(
        &mut self,
        event: &TimedEvent,
        event_id: Option<usize>,
        config: &LevelConfig,
    ) {
        if self.time < event.time {
            self.is_finished = false;
            return;
        }

        if let Some(time) = self.ignore_after {
            if event.time > time {
                return;
            }
        }

        let time = self.time - event.time;

        match &event.event {
            Event::PaletteSwap => self.swap_palette = !self.swap_palette,
            Event::Light(event) => {
                let (telegraph, light) = render_light(event, time, event_id, config);
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
    config: &LevelConfig,
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

        if config.waypoints.show {
            let sustain_time = seconds_to_time(config.waypoints.sustain_time);
            let fade_time = seconds_to_time(config.waypoints.fade_time);
            let sustain_scale = config.waypoints.sustain_scale;

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
                            let t = r32(relative_time as f32 / sustain_time as f32);
                            sustain_scale
                                + crate::util::smoothstep(FloatTime::ONE - t)
                                    * (Coord::ONE - sustain_scale)
                        } else {
                            let t = r32((relative_time - sustain_time) as f32 / fade_time as f32);
                            sustain_scale * crate::util::smoothstep(FloatTime::ONE - t)
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
        } else {
            vec![main_tele]
        }
    };

    // Light
    let relative_time = relative_time - event.telegraph.precede_time;
    let light = (relative_time > Time::ZERO && relative_time < duration).then(|| {
        let transform = event.light.movement.get(relative_time);
        let mut main_light = base_tele.light;
        main_light.collider = base_light.collider.transformed(transform);
        let (id, _, closest_time) = event.light.movement.closest_waypoint(relative_time);
        main_light.closest_waypoint = (closest_time - relative_time, id);
        main_light
    });

    (telegraphs, light)
}
