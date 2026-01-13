use super::*;

pub struct GameplayPreview {
    pub camera: Camera2d,
    pub level: Level,
    pub state: LevelState,
    pub player: Player,
    pub real_time: FloatTime,
    pub render_time: Time,
}

impl GameplayPreview {
    pub fn new() -> Self {
        let level = infinity_level();
        Self {
            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: Camera2dFov::Cover {
                    width: 8.0,
                    height: 6.0,
                    scale: 1.0,
                },
            },
            state: LevelState::render(&level, 0, None, None),
            level,
            player: Player::new(Collider::new(vec2::ZERO, Shape::circle(r32(0.5))), r32(1.0)),
            real_time: FloatTime::ZERO,
            render_time: 0,
        }
    }

    pub fn update(&mut self, delta_time: FloatTime) {
        self.real_time += delta_time;
        if let Some(event) = self.level.events.first()
            && let Event::Light(light) = &event.event
        {
            let duration = light.movement.duration()
                - light.movement.get_fade_in()
                - light.movement.get_fade_out();
            self.render_time =
                light.movement.get_fade_in() + seconds_to_time(self.real_time) % duration;
        }

        self.state = LevelState::render(&self.level, self.render_time, None, None);

        if let Some(light) = self.state.lights.first() {
            self.player.collider.position = light.collider.position;
        }
        for light in &self.state.lights {
            self.player.update_distance_simple(&light.collider);
        }
        self.player.update_tail(delta_time);
    }
}

fn infinity_level() -> Level {
    let bpm = r32(150.0);
    let beat_time = r32(60.0) / bpm;
    let lerp_time = seconds_to_time(beat_time);
    let interpolation = MoveInterpolation::Linear;
    const X_MID: f32 = 1.0;
    const X_MAX: f32 = 1.7;
    const Y: f32 = 1.2;
    Level {
        events: vec![TimedEvent {
            time: 0,
            event: Event::Light(LightEvent {
                danger: false,
                shape: Shape::circle(r32(1.3)),
                movement: Movement {
                    initial: WaypointInitial {
                        lerp_time,
                        interpolation,
                        curve: TrajectoryInterpolation::Bezier,
                        transform: TransformLight::default(),
                    },
                    waypoints: vec![
                        Waypoint {
                            lerp_time,
                            interpolation,
                            change_curve: None,
                            transform: TransformLight {
                                translation: vec2(-X_MID, Y).as_r32(),
                                ..default()
                            },
                        },
                        Waypoint {
                            lerp_time,
                            interpolation,
                            change_curve: Some(TrajectoryInterpolation::Bezier),
                            transform: TransformLight {
                                translation: vec2(-X_MAX, 0.0).as_r32(),
                                ..default()
                            },
                        },
                        Waypoint {
                            lerp_time,
                            interpolation,
                            change_curve: None,
                            transform: TransformLight {
                                translation: vec2(-X_MID, -Y).as_r32(),
                                ..default()
                            },
                        },
                        Waypoint {
                            lerp_time,
                            interpolation,
                            change_curve: Some(TrajectoryInterpolation::Bezier),
                            transform: TransformLight::default(),
                        },
                        Waypoint {
                            lerp_time,
                            interpolation,
                            change_curve: None,
                            transform: TransformLight {
                                translation: vec2(X_MID, Y).as_r32(),
                                ..default()
                            },
                        },
                        Waypoint {
                            lerp_time,
                            interpolation,
                            change_curve: Some(TrajectoryInterpolation::Bezier),
                            transform: TransformLight {
                                translation: vec2(X_MAX, 0.0).as_r32(),
                                ..default()
                            },
                        },
                        Waypoint {
                            lerp_time,
                            interpolation,
                            change_curve: None,
                            transform: TransformLight {
                                translation: vec2(X_MID, -Y).as_r32(),
                                ..default()
                            },
                        },
                        Waypoint {
                            lerp_time,
                            interpolation,
                            change_curve: Some(TrajectoryInterpolation::Bezier),
                            transform: TransformLight::default(),
                        },
                        Waypoint {
                            lerp_time,
                            interpolation,
                            change_curve: None,
                            transform: TransformLight {
                                translation: vec2(-X_MID, Y).as_r32(),
                                ..default()
                            },
                        },
                    ]
                    .into(),
                    last: TransformLight {
                        translation: vec2(-X_MAX, 0.0).as_r32(),
                        ..default()
                    },
                },
            }),
        }],
        timing: Timing::new(bpm),
    }
}
