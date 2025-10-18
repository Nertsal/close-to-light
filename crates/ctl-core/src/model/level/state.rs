use super::*;

/// A renderable state of the level at some given time.
#[derive(Debug)]
pub struct LevelState {
    /// The time at which the render has been done.
    time: Time,
    /// The time after which events are not rendered.
    ignore_after: Option<Time>,
    timing: Timing,
    pub lights: Vec<Light>,
    pub telegraphs: Vec<LightTelegraph>,
    pub is_finished: bool,
}

impl Default for LevelState {
    fn default() -> Self {
        Self {
            time: Time::ZERO,
            ignore_after: None,
            timing: Timing::default(),
            lights: Vec::new(),
            telegraphs: Vec::new(),
            is_finished: false,
        }
    }
}

impl LevelState {
    /// The time at which the render has been done.
    pub fn time(&self) -> Time {
        self.time
    }

    pub fn render(
        level: &Level,
        config: &LevelConfig,
        time: Time,
        ignore_after: Option<Time>,
        mut vfx: Option<&mut Vfx>,
    ) -> Self {
        let mut state = Self {
            time,
            ignore_after,
            timing: level.timing.clone(),
            lights: Vec::new(),
            telegraphs: Vec::new(),
            is_finished: true,
        };

        if let Some(vfx) = &mut vfx {
            // Reset accumulative fields
            vfx.palette_swap.target = R32::ZERO;
            vfx.rgb_split.time_left = FloatTime::ZERO;
            vfx.camera_shake = R32::ZERO;
        }

        for (i, e) in level.events.iter().enumerate() {
            state.render_event(e, Some(i), config, vfx.as_deref_mut());
        }

        if state.is_finished
            && let Some(vfx) = vfx
        {
            // Reset persistent vfx
            vfx.palette_swap.target = R32::ZERO;
        }

        state
    }

    pub fn render_event(
        &mut self,
        event: &TimedEvent,
        event_id: Option<usize>,
        config: &LevelConfig,
        vfx: Option<&mut Vfx>,
    ) {
        if let Some(time) = self.ignore_after
            && event.time > time
        {
            return;
        }

        let time = self.time - event.time;
        if time < 0 {
            // The event is in the future
            self.is_finished = false;
        }

        match &event.event {
            Event::Light(light) => {
                let precede_time = seconds_to_time(self.timing.get_timing(event.time).beat_time);
                if self.time < event.time - precede_time {
                    return;
                }
                let (telegraph, light) = render_light(light, time, event_id, config, precede_time);
                self.telegraphs.extend(telegraph);
                self.lights.extend(light);
            }
            Event::Effect(effect) => match *effect {
                EffectEvent::PaletteSwap(duration) => {
                    if self.time < event.time {
                        return;
                    }
                    if let Some(vfx) = vfx {
                        let t = (time as f32 / duration as f32).clamp(0.0, 1.0);
                        vfx.palette_swap.target = if vfx.palette_swap.target > r32(0.5) {
                            r32(1.0 - t)
                        } else {
                            r32(t)
                        };
                    }
                }
                EffectEvent::RgbSplit(duration) => {
                    if self.time < event.time || self.time > event.time + duration {
                        return;
                    }
                    if let Some(vfx) = vfx {
                        vfx.rgb_split.time_left = time_to_seconds(duration - time);
                    }
                }
                EffectEvent::CameraShake(duration, intensity) => {
                    if self.time < event.time || self.time > event.time + duration {
                        return;
                    }
                    if let Some(vfx) = vfx {
                        vfx.camera_shake = intensity;
                    }
                }
            },
        }

        self.is_finished = self.is_finished && self.lights.is_empty() && self.telegraphs.is_empty();
    }
}

pub fn render_light(
    event: &LightEvent,
    relative_time: Time,
    event_id: Option<usize>,
    config: &LevelConfig,
    precede_time: Time,
) -> (Vec<LightTelegraph>, Option<Light>) {
    let movement = &event.movement;
    let base_light = event.clone().instantiate(event_id);
    let base_tele = base_light.clone().into_telegraph();
    let duration = event.movement.total_duration();

    // Light
    let light = (relative_time > Time::ZERO && relative_time < duration).then(|| {
        let transform = event.movement.get(relative_time);
        let mut main_light = base_tele.light.clone();
        main_light.collider = base_light.collider.transformed(transform);
        let (id, _, closest_time) = event.movement.closest_waypoint(relative_time);
        main_light.closest_waypoint = (closest_time - relative_time, id);
        main_light
    });

    // Telegraph
    let relative_time = relative_time + precede_time;
    let telegraphs = if relative_time > duration {
        vec![]
    } else {
        let transform = event.movement.get(relative_time);
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

    (telegraphs, light)
}
