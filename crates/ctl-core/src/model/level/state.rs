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
    pub waypoints: Vec<LightWaypoint>,
    pub is_finished: bool,
}

#[derive(Debug)]
pub struct LightWaypoint {
    /// The time at which the light will reach this waypoint.
    pub time: Time,
    pub light: Light,
}

impl Default for LevelState {
    fn default() -> Self {
        Self {
            time: Time::ZERO,
            ignore_after: None,
            timing: Timing::default(),
            lights: Vec::new(),
            telegraphs: Vec::new(),
            waypoints: Vec::new(),
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
            waypoints: Vec::new(),
            is_finished: true,
        };

        if let Some(vfx) = &mut vfx {
            // Reset accumulative fields
            vfx.reset();
        }

        for (i, e) in level.events.iter().enumerate() {
            state.render_event(e, Some(i), vfx.as_deref_mut());
        }

        if state.is_finished
            && let Some(vfx) = vfx
        {
            // Reset persistent vfx
            vfx.reset();
        }

        state
    }

    pub fn render_event(
        &mut self,
        event: &TimedEvent,
        event_id: Option<usize>,
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
                let timing = self.timing.get_timing(event.time);
                let precede_time = seconds_to_time(timing.beat_time);
                if self.time < event.time - precede_time {
                    return;
                }
                let (light, telegraph, waypoints) =
                    render_light(light, time, event_id, precede_time, &timing);
                self.lights.extend(light);
                self.telegraphs.extend(telegraph);
                self.waypoints.extend(waypoints);
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
    precede_time: Time,
    timing: &TimingPoint,
) -> (Option<Light>, Option<LightTelegraph>, Vec<LightWaypoint>) {
    let movement = &event.movement;
    let base_light = event.clone().instantiate(event_id);
    let base_tele = base_light.clone().into_telegraph();
    let duration = event.movement.duration();

    // Light
    let light = (relative_time > Time::ZERO && relative_time < duration).then(|| {
        let transform = event.movement.get(relative_time);
        let mut main_light = base_tele.light.clone();
        main_light.collider = base_light.collider.transformed(transform);
        let (id, _, closest_time) = event.movement.closest_waypoint(relative_time);
        main_light.closest_waypoint = (closest_time - relative_time, id);
        main_light.hollow = transform.hollow;
        main_light
    });

    // Telegraph
    let relative_time = relative_time + precede_time;
    let (telegraph, waypoints) = if relative_time > duration {
        (None, vec![])
    } else {
        let transform = event.movement.get(relative_time);
        let mut main_tele = base_tele.clone();
        main_tele.light.collider = base_light.collider.transformed(transform);

        let mut last_pos = movement.initial.transform.translation;
        let waypoints = movement
            .timed_transforms()
            .take(movement.waypoints.len()) // Ignore the last position
            .skip(1) // Ignore the initial position
            .filter_map(|(_, transform, time)| {
                let relative_time = relative_time - time;

                // TODO: move these constants into some config
                let radius_max = 0.2;
                let width = 0.05;
                let beat_time = timing.beat_time.as_f32();
                let fade_in = 0.5 * beat_time;
                let fade_out = 0.75 * beat_time;

                let t = time_to_seconds(relative_time).as_f32();
                let t = (t / fade_in + 1.0).min(1.0 - t / fade_out).max(0.0);
                let radius = r32(t * radius_max);

                let waypoint = (last_pos != transform.translation
                    && (Time::ZERO..seconds_to_time(r32(fade_in + fade_out)))
                        .contains(&relative_time)
                    && radius.as_f32() > width)
                    .then(|| {
                        let shape = match base_light.collider.shape {
                            Shape::Circle { .. } => Shape::circle(radius),
                            Shape::Line { .. } => Shape::line(radius / r32(2.0)),
                            Shape::Rectangle { .. } => Shape::rectangle(vec2::splat(radius)),
                        };
                        let collider = Collider {
                            shape,
                            ..base_light.collider.transformed(transform)
                        };
                        let mut light = base_light.clone();
                        light.collider = collider;
                        LightWaypoint {
                            time: relative_time - time,
                            light,
                        }
                    });
                last_pos = transform.translation;
                waypoint
            });

        (Some(main_tele), waypoints.collect())
    };

    (light, telegraph, waypoints)
}
