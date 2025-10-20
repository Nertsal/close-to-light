mod event;

use super::*;

#[derive(Debug, Clone)]
pub enum ParticleDistribution {
    Circle {
        center: vec2<Coord>,
        radius: Coord,
    },
    Quad {
        aabb: Aabb2<Coord>,
        angle: Angle<Coord>,
    },
}

impl ParticleDistribution {
    pub fn sample(&self, rng: &mut impl Rng, density: R32) -> Vec<vec2<Coord>> {
        match *self {
            ParticleDistribution::Quad { aabb, angle } => {
                let amount = density * aabb.width() * aabb.height();
                let extra = if rng.gen_bool(amount.fract().as_f32().into()) {
                    1
                } else {
                    0
                };
                let amount = (amount.floor()).as_f32() as usize + extra;

                (0..amount)
                    .map(|_| {
                        (vec2(
                            rng.gen_range(aabb.min.x..=aabb.max.x),
                            rng.gen_range(aabb.min.y..=aabb.max.y),
                        ) - aabb.center())
                        .rotate(angle)
                            + aabb.center()
                    })
                    .collect()
            }
            ParticleDistribution::Circle { center, radius } => {
                let amount = density * radius.sqr() * R32::PI;
                let extra = if rng.gen_bool(amount.fract().as_f32().into()) {
                    1
                } else {
                    0
                };
                let amount = (amount.floor()).as_f32() as usize + extra;

                (0..amount)
                    .map(|_| rng.gen_circle(center, radius))
                    .collect()
            }
        }
    }
}

impl Model {
    /// Initialize the level by playing the events from the negative time.
    pub fn init(&mut self, target_time: Time) {
        log::info!("Starting at the requested time {target_time}...");

        // NOTE: double conversion in case of floating point errors
        // since `play_time` is used to recalculate `play_time_ms` every frame
        self.play_time = time_to_seconds(target_time);
        self.play_time_ms = seconds_to_time(self.play_time) - self.music_offset;
        self.completion_time = self.play_time;

        self.player.health.set_ratio(FloatTime::ONE);
        self.state = State::Starting {
            start_timer: r32(1.0),
            music_start_time: target_time,
        };
    }

    pub fn update(&mut self, player_target: vec2<Coord>, delta_time: FloatTime) {
        let options = self.context.get_options();
        self.context.music.set_volume(options.volume.music());
        self.vfx.update(delta_time);

        // Camera shake
        self.camera.center = self.camera.center * 0.5
            + Angle::from_degrees(thread_rng().gen_range(0.0..=360.0)).unit_vec()
                * self.vfx.camera_shake.as_f32();

        let delta_ms = seconds_to_time(delta_time);
        self.update_rhythm(delta_ms);

        if let Some(button) = &mut self.transition_button {
            button.update(true, delta_time);
            if button.hover_time.is_max() {
                self.transition_button = None;
            }
        }

        // Move
        self.player.collider.position = player_target;

        if let State::Starting { .. } = self.state {
        } else {
            self.play_time += delta_time;
            self.play_time_ms = seconds_to_time(self.play_time) - self.music_offset;
        }
        if let State::Playing = self.state {
            self.completion_time += delta_time;
        }

        self.real_time += delta_time;
        self.switch_time += delta_time;

        if let State::Lost { .. } = self.state {
            let t = 1.0 - self.switch_time.as_f32() / 2.0;
            let speed = (t - 0.1).max(0.5);
            self.context.music.set_speed(speed);

            let volume = t * options.volume.music();
            if volume < 0.0 {
                self.context.music.stop();
            } else {
                self.context.music.set_volume(volume);
            }
        }

        // Update level state
        let ignore_time = match self.state {
            State::Lost {
                death_time_ms: death_beat_time,
            } => Some(death_beat_time),
            _ => None,
        };
        self.level_state = LevelState::render(
            &self.level.level.data,
            &self.level.config,
            self.play_time_ms,
            ignore_time,
            Some(&mut self.vfx),
        );

        // Update fire
        for (pos, size, _) in &mut self.fire {
            *pos += vec2(0.0, 2.0).as_r32() * delta_time;
            *size -= r32(0.5) * delta_time;
        }
        self.fire.retain(|(_, size, _)| size.as_f32() > 0.0);
        let mut rng = thread_rng();
        for light in &self.level_state.lights {
            let cover = r32(0.7);
            let (size, shape) = match light.collider.shape {
                Shape::Circle { radius } => (
                    radius,
                    ParticleDistribution::Circle {
                        center: light.collider.position,
                        radius: radius * cover,
                    },
                ),
                Shape::Line { width } => (
                    width,
                    ParticleDistribution::Quad {
                        aabb: Aabb2::point(light.collider.position)
                            .extend_symmetric(vec2(r32(20.0), width * r32(0.25) * cover)),
                        angle: light.collider.rotation,
                    },
                ),
                Shape::Rectangle { width, height } => (
                    width.min(height),
                    ParticleDistribution::Quad {
                        aabb: Aabb2::point(light.collider.position)
                            .extend_symmetric(vec2(width, height) * r32(0.25) * cover),
                        angle: light.collider.rotation,
                    },
                ),
            };
            let pos = shape.sample(&mut rng, r32(1.0));
            let size = size * r32(0.3);
            self.fire
                .extend(pos.into_iter().map(|pos| (pos, size, light.danger)));
        }

        // Update player's light state
        // And check for missed rhythm
        let get_light = |id: Option<usize>, pass: bool| {
            id.and_then(|id| {
                self.level_state
                    .lights
                    .iter()
                    .find(|light| light.event_id == Some(id))
                    .filter(|light| {
                        // Can only miss after the waypoint, not before, hence no buffer time
                        // (allows for unpunished early exit)
                        //
                        // A miss occurs when the player was inside a light that was leaving its waypoint
                        // and has missed the coyote time
                        //
                        // `pass` used to extend the coyote time for `last_light`,
                        // because otherwise we cannot detect a miss
                        // as both will get set to `None` at the same frame
                        // (allows for unpunished late entrance)
                        let time = light.closest_waypoint.0;
                        time < 0 && (time > -COYOTE_TIME || pass && time > -COYOTE_TIME * 2)
                    })
                    .map(|light| (id, light.closest_waypoint.1))
            })
        };
        let last_light = get_light(self.player.closest_light, true);

        // Update light state
        self.player.reset_distance();
        for light in self.level_state.lights.iter() {
            self.player.update_light_distance(light, self.last_rhythm);
        }

        // Check missed rhythm
        let light = get_light(self.player.closest_light, false);
        if last_light.is_some() && last_light != light && last_light != Some(self.last_rhythm) {
            // Light has changed and no perfect rhythm
            self.score.metrics.discrete.missed_rhythm();
            self.handle_event(GameEvent::Rhythm { perfect: false });
        }

        match &mut self.state {
            State::Starting {
                start_timer,
                music_start_time,
            } => {
                let music_start_time = *music_start_time;
                *start_timer -= delta_time;
                if *start_timer <= FloatTime::ZERO && self.player.is_perfect {
                    self.start(music_start_time);
                }
            }
            State::Playing => {
                if self.level_state.is_finished {
                    self.finish();
                } else if !self.level.config.modifiers.clean_auto {
                    // Player health
                    if let Some(distance) = self.player.danger_distance {
                        let multiplier = (r32(1.0) - distance + r32(0.5)).min(r32(1.0));
                        self.player.health.change(
                            -self.level.config.health.danger_decrease_rate
                                * multiplier
                                * delta_time,
                        );
                    } else if self.player.light_distance.is_some() {
                        self.player
                            .health
                            .change(self.level.config.health.restore_rate * delta_time);
                    } else {
                        self.player
                            .health
                            .change(-self.level.config.health.dark_decrease_rate * delta_time);
                    }

                    let perfect_rhythm = self.score.update(&self.player, delta_time);
                    if perfect_rhythm {
                        self.handle_event(GameEvent::Rhythm { perfect: true });
                    }

                    if !self.level.config.modifiers.nofail && self.player.health.is_min() {
                        self.lose();
                    }
                }
            }
            _ if self.switch_time > FloatTime::ONE => {
                // 1 second before the UI is active
                let hovering = self
                    .restart_button
                    .base_collider
                    .check(&self.player.collider);
                if hovering && self.cursor_clicked {
                    self.restart_button.clicked = true;
                }
                self.restart_button.update(hovering, delta_time);
                self.player
                    .update_distance_simple(&self.restart_button.base_collider);
                if self.restart_button.is_fading() {
                    self.restart();
                }

                // 1 second before the UI is active
                let hovering = self.exit_button.base_collider.check(&self.player.collider);
                if hovering && self.cursor_clicked {
                    self.exit_button.clicked = true;
                }
                self.exit_button.update(hovering, delta_time);
                self.player
                    .update_distance_simple(&self.exit_button.base_collider);
                if self.exit_button.hover_time.is_max() {
                    self.transition = Some(Transition::Exit);
                }
            }
            _ => (),
        }

        if !self.level.config.modifiers.clean_auto {
            // Player tail
            self.player.update_tail(delta_time);
        }
    }

    fn update_rhythm(&mut self, delta_time: Time) {
        for rhythm in &mut self.rhythms {
            rhythm.time.change(delta_time);
        }
        self.rhythms.retain(|rhythm| !rhythm.time.is_max());
    }

    fn restart(&mut self) {
        log::info!("Restarting...");
        *self = Self::new(
            self.context.clone(),
            PlayLevel {
                transition_button: Some(self.restart_button.clone()),
                ..self.level.clone()
            },
            self.leaderboard.clone(),
        );
    }

    pub fn start(&mut self, music_start_time: Time) {
        self.state = State::Playing;
        if let Some(music) = &self.level.group.music {
            log::debug!("Starting music at {music_start_time}");
            self.context.music.play_from_time(music, music_start_time);
        }
    }

    pub fn finish(&mut self) {
        self.state = State::Finished;
        self.context.music.stop();
        self.switch_time = FloatTime::ZERO;
        self.get_leaderboard(true);
    }

    pub fn lose(&mut self) {
        self.state = State::Lost {
            death_time_ms: self.play_time_ms,
        };
        self.switch_time = FloatTime::ZERO;
        self.get_leaderboard(true);
    }

    pub fn get_leaderboard(&mut self, submit_score: bool) {
        self.transition = Some(Transition::LoadLeaderboard { submit_score });
    }

    /// Calculates the current completion percentage (in range 0..=1).
    pub fn current_completion(&self) -> R32 {
        let t = self.completion_time / time_to_seconds(self.level.level.data.last_time());
        t.clamp(R32::ZERO, R32::ONE)
    }
}
