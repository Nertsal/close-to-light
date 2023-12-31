use super::*;

impl Model {
    /// Initialize the level by playing the events from the negative time.
    pub fn init(&mut self, target_time: Time) {
        log::info!("Starting at the requested time {:.2}...", target_time);
        self.beat_time = target_time / self.music.beat_time();
        self.player.health.set_ratio(Time::ONE);
        self.state = State::Starting {
            start_timer: r32(1.0),
            music_start_time: target_time,
        };
    }

    pub fn update(&mut self, player_target: vec2<Coord>, delta_time: Time) {
        self.music.set_volume(self.options.volume.music());

        // Move
        self.player.collider.position = player_target;

        if let State::Starting { .. } = self.state {
        } else {
            self.beat_time += delta_time / self.music.beat_time();
        }

        self.real_time += delta_time;
        self.switch_time += delta_time;

        if let State::Lost { .. } = self.state {
            if let Some(music) = &mut self.music.effect {
                let t = 1.0 - self.switch_time.as_f32() / 2.0;
                let speed = (t - 0.1).max(0.5);
                music.set_speed(speed as f64);

                let volume = t;
                if volume < 0.0 {
                    self.music.stop();
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
        self.level_state =
            LevelState::render(&self.level, &self.config, self.beat_time, ignore_time);

        // Check if the player is in light
        self.player.reset_distance();
        for light in self.level_state.lights.iter() {
            self.player.update_distance(&light.collider, light.danger);
        }

        match &mut self.state {
            State::Starting {
                start_timer,
                music_start_time,
            } => {
                let music_start_time = *music_start_time;
                *start_timer -= delta_time;
                if *start_timer <= Time::ZERO && self.player.light_distance.is_some() {
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
                } else if !self.config.modifiers.clean_auto {
                    // Player health
                    if let Some(distance) = self.player.danger_distance {
                        let multiplier = (r32(1.0) - distance + r32(0.5)).min(r32(1.0));
                        self.player.health.change(
                            -self.config.health.danger_decrease_rate * multiplier * delta_time,
                        );
                    } else if let Some(distance) = self.player.light_distance {
                        let distance = distance.clamp(r32(0.0), r32(1.3)) / r32(1.3);
                        let score_multiplier = (r32(1.0) - distance + r32(0.5)).min(r32(1.0));
                        self.player
                            .health
                            .change(self.config.health.restore_rate * delta_time);
                        self.score += delta_time * score_multiplier * r32(100.0);
                    } else {
                        self.player
                            .health
                            .change(-self.config.health.dark_decrease_rate * delta_time);
                    }

                    if !self.config.modifiers.nofail && self.player.health.is_min() {
                        self.lose();
                    }
                }
            }
            _ if self.switch_time > Time::ONE => {
                // 1 second before the UI is active
                let hovering = self
                    .restart_button
                    .base_collider
                    .check(&self.player.collider);
                self.restart_button.update(hovering, delta_time);
                self.player
                    .update_distance(&self.restart_button.base_collider, false);
                if self.restart_button.hover_time.is_max() {
                    self.restart();
                }

                // 1 second before the UI is active
                let hovering = self.exit_button.base_collider.check(&self.player.collider);
                self.exit_button.update(hovering, delta_time);
                self.player
                    .update_distance(&self.exit_button.base_collider, false);
                if self.exit_button.hover_time.is_max() {
                    self.transition = Some(Transition::Exit);
                }
            }
            _ => (),
        }

        if !self.config.modifiers.clean_auto {
            // Player tail
            self.player.update_tail(delta_time);
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
            self.options.clone(),
            self.config.clone(),
            self.level.clone(),
            self.music.clone(),
            self.secrets.clone(),
            self.player.name.clone(),
            Time::ZERO,
        );
    }

    pub fn start(&mut self, music_start_time: Time) {
        self.state = State::Playing;
        self.music.play_from(time::Duration::from_secs_f64(
            music_start_time.as_f32() as f64
        ));
    }

    pub fn finish(&mut self) {
        self.save_highscore();
        self.state = State::Finished;
        self.music.stop();
        self.switch_time = Time::ZERO;
        self.get_leaderboard(true);
    }

    pub fn lose(&mut self) {
        self.save_highscore();
        self.state = State::Lost {
            death_beat_time: self.beat_time,
        };
        self.switch_time = Time::ZERO;
        self.get_leaderboard(false);
    }

    pub fn get_leaderboard(&mut self, submit_score: bool) {
        self.transition = Some(Transition::LoadLeaderboard { submit_score });
    }
}
