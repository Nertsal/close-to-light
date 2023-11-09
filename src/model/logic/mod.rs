use super::*;

impl Model {
    /// Initialize the level by playing the events from the negative time.
    pub fn init(&mut self, target_time: Time) {
        log::info!("Starting at the requested time {:.2}...", target_time);
        self.beat_time = target_time / self.level.beat_time();
        self.player.health.set_ratio(Time::ONE);
        self.state = State::Starting {
            start_timer: r32(1.0),
            music_start_time: target_time,
        };
    }

    pub fn update(&mut self, player_target: vec2<Coord>, delta_time: Time) {
        // Move
        self.player.collider.position = player_target;

        if let State::Starting { .. } = self.state {
        } else {
            self.beat_time += delta_time / self.level.beat_time();
        }

        self.real_time += delta_time;
        self.switch_time += delta_time;

        // Player tail
        self.player.update_tail(delta_time);

        if let State::Lost { .. } = self.state {
            if let Some(music) = &mut self.music {
                let speed = (1.0 - self.switch_time.as_f32() / 0.5).max(0.0) + 0.5;
                music.set_speed(speed as f64);

                let volume = 1.0 - self.switch_time.as_f32() / 5.0;
                if volume < 0.0 {
                    self.stop_music();
                } else {
                    music.set_volume(volume as f64);
                }
            }
        }

        // Update level state
        let ignore_time = match self.state {
            State::Lost { death_beat_time } => Some(death_beat_time),
            _ => None,
        };
        self.level_state = LevelState::render(&self.level, self.beat_time, ignore_time);

        // Check if the player is in light
        let mut distance_normalized: Option<R32> = None;
        let mut distance_danger_normalized: Option<R32> = None;
        for light in self.level_state.lights.iter() {
            let delta_pos = self.player.collider.position - light.collider.position;
            let distance = match light.base_collider.shape {
                Shape::Circle { radius } => delta_pos.len() / radius,
                Shape::Line { width } => {
                    let dir = light.collider.rotation.unit_vec();
                    let dir = vec2(-dir.y, dir.x); // perpendicular
                    let dot = dir.x * delta_pos.x + dir.y * delta_pos.y;
                    dot.abs() / (width / r32(2.0))
                }
                Shape::Rectangle { .. } => todo!(),
            };

            if light.danger {
                distance_danger_normalized =
                    Some(distance.min(distance_danger_normalized.unwrap_or(distance)));
            } else {
                distance_normalized = Some(distance.min(distance_normalized.unwrap_or(distance)));
            }
        }
        self.player.light_distance_normalized = match distance_normalized {
            Some(distance) if distance <= r32(1.0) => Some(distance),
            _ => None,
        };
        self.player.danger_distance_normalized = match distance_danger_normalized {
            Some(distance) if distance <= r32(1.0) => Some(distance),
            _ => None,
        };

        match &mut self.state {
            State::Starting {
                start_timer,
                music_start_time,
            } => {
                let music_start_time = *music_start_time;
                *start_timer -= delta_time;
                if *start_timer <= Time::ZERO && self.player.light_distance_normalized.is_some() {
                    self.start(music_start_time);
                }
            }
            State::Playing => {
                if self.level_state.is_finished {
                    // if self.level.rng_end {
                    //     // No more events - start rng
                    //     let telegraph = self.random_light_telegraphed();
                    //     self.telegraphs.push(telegraph);
                    // } else
                    self.finish();
                } else {
                    if let Some(distance) = self.player.danger_distance_normalized {
                        let multiplier = (r32(1.0) - distance + r32(0.5)).min(r32(1.0));
                        self.player.health.change(
                            -self.level_state.config.health.danger_decrease_rate
                                * multiplier
                                * delta_time,
                        );
                    } else if let Some(distance) = self.player.light_distance_normalized {
                        let score_multiplier = (r32(1.0) - distance + r32(0.5)).min(r32(1.0));
                        self.player
                            .health
                            .change(self.level_state.config.health.restore_rate * delta_time);
                        self.score += delta_time * score_multiplier * r32(100.0);
                    } else {
                        self.player.health.change(
                            -self.level_state.config.health.dark_decrease_rate * delta_time,
                        );
                    }

                    if self.player.health.is_min() {
                        self.lose();
                    }
                }
            }
            _ if self.switch_time > Time::ONE => {
                // 1 second before the UI is active
                self.restart_button.hover_time.change(
                    if self.restart_button.collider.check(&self.player.collider) {
                        delta_time
                    } else {
                        -delta_time
                    },
                );
                if self.restart_button.hover_time.is_max() {
                    self.restart();
                }
            }
            _ => (),
        }
    }

    pub fn save_highscore(&self) {
        let high_score = self.high_score.max(self.score);
        preferences::save("highscore", &high_score);
    }

    fn restart(&mut self) {
        log::info!("Restarting...");
        self.save_highscore();
        *self = Self::new(
            &self.assets,
            self.config.clone(),
            self.level.clone(),
            self.secrets.clone(),
            self.player.name.clone(),
            Time::ZERO,
        );
    }

    pub fn start(&mut self, music_start_time: Time) {
        self.state = State::Playing;
        self.stop_music();
        let mut music = self.assets.music.effect();
        music.play_from(time::Duration::from_secs_f64(
            music_start_time.as_f32() as f64
        ));
        self.music = Some(music);
    }

    pub fn finish(&mut self) {
        self.save_highscore();
        self.state = State::Finished;
        self.stop_music();
        self.switch_time = Time::ZERO;
        self.get_leaderboard(true);
    }

    pub fn lose(&mut self) {
        self.save_highscore();
        self.state = State::Lost {
            death_beat_time: self.beat_time,
        };
        // if let Some(music) = &mut self.music {
        //     music.set_speed(0.5);
        //     music.set_volume(1.2); // Compensate for lower speed being quiter
        // }
        self.switch_time = Time::ZERO;
        self.get_leaderboard(false);
    }

    pub fn get_leaderboard(&mut self, submit_score: bool) {
        self.transition = Some(Transition::LoadLeaderboard { submit_score });
    }

    // fn random_light_telegraphed(&self) -> LightTelegraph {
    //     self.random_light().into_telegraph(
    //         Telegraph {
    //             precede_time: r32(1.0),
    //             speed: r32(1.0),
    //         },
    //         self.level.beat_time(),
    //     )
    // }

    // fn random_light(&self) -> Light {
    //     let mut rng = thread_rng();

    //     let position = vec2(rng.gen_range(-5.0..=5.0), rng.gen_range(-5.0..=5.0)).as_r32();
    //     let rotation = Angle::from_degrees(r32(rng.gen_range(0.0..=360.0)));

    //     let shape = *self
    //         .config
    //         .shapes
    //         .choose(&mut rng)
    //         .expect("no shapes available");

    //     let collider = Collider {
    //         position,
    //         rotation,
    //         shape,
    //     };
    //     Light {
    //         base_collider: collider.clone(),
    //         collider,
    //         movement: Movement {
    //             key_frames: vec![
    //                 MoveFrame {
    //                     lerp_time: 0.0,
    //                     transform: Transform {
    //                         scale: r32(0.0),
    //                         ..default()
    //                     },
    //                 },
    //                 MoveFrame {
    //                     lerp_time: 1.0,
    //                     transform: Transform::identity(),
    //                 },
    //                 MoveFrame {
    //                     lerp_time: 1.0,
    //                     transform: Transform {
    //                         scale: r32(0.0),
    //                         ..default()
    //                     },
    //                 },
    //             ]
    //             .into(),
    //         }
    //         .with_beat_time(self.level.beat_time()),
    //         lifetime: Time::ZERO,
    //         // lifetime: Lifetime::new_max(r32(2.0) * self.level.beat_time()),
    //     }
    // }
}
