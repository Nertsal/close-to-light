mod event;

use super::*;

impl Model {
    /// Initialize the level by playing the events from the negative time.
    pub fn init(&mut self, target_time: Time) {
        log::info!("Starting at the requested time {}...", target_time);
        self.exact_time = target_time;
        self.player.health.set_ratio(FloatTime::ONE);
        self.state = State::Starting {
            start_timer: r32(1.0),
            music_start_time: target_time,
        };
    }

    pub fn update(&mut self, player_target: vec2<Coord>, delta_time: FloatTime) {
        self.context.music.set_volume(self.options.volume.music());

        let exact_delta = seconds_to_time(delta_time);
        self.update_rhythm(exact_delta);

        // Move
        self.player.collider.position = player_target;

        if let State::Starting { .. } = self.state {
        } else {
            self.exact_time += exact_delta;
        }

        self.real_time += delta_time;
        self.switch_time += delta_time;

        if let State::Lost { .. } = self.state {
            let t = 1.0 - self.switch_time.as_f32() / 2.0;
            let speed = (t - 0.1).max(0.5);
            self.context.music.set_speed(speed);

            let volume = t * self.options.volume.music();
            if volume < 0.0 {
                self.context.music.stop();
            } else {
                self.context.music.set_volume(volume);
            }
        }

        // Update level state
        let ignore_time = match self.state {
            State::Lost {
                death_exact_time: death_beat_time,
            } => Some(death_beat_time),
            _ => None,
        };
        self.level_state = LevelState::render(
            &self.level.level.data,
            &self.level.config,
            self.exact_time,
            ignore_time,
        );

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
                    // if self.level.rng_end {
                    //     // No more events - start rng
                    //     let telegraph = self.random_light_telegraphed();
                    //     self.telegraphs.push(telegraph);
                    // } else
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

                    let events = self.score.update(&self.player, delta_time);
                    for event in events {
                        self.handle_event(event);
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
                if self.restart_button.hover_time.is_max() {
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

    pub fn save_highscore(&self) {
        let high_score = self.high_score.max(self.score.calculated.combined);
        preferences::save("highscore", &high_score);
    }

    fn restart(&mut self) {
        log::info!("Restarting...");
        self.save_highscore();
        *self = Self::new(
            self.context.clone(),
            self.options.clone(),
            self.level.clone(),
            self.leaderboard.clone(),
        );
    }

    pub fn start(&mut self, music_start_time: Time) {
        self.state = State::Playing;
        if let Some(music) = &self.level.group.music {
            log::debug!("Starting music at {}", music_start_time);
            self.context.music.play_from_time(music, music_start_time);
        }
    }

    pub fn finish(&mut self) {
        self.save_highscore();
        self.state = State::Finished;
        self.context.music.stop();
        self.switch_time = FloatTime::ZERO;
        self.get_leaderboard(true);
    }

    pub fn lose(&mut self) {
        self.save_highscore();
        self.state = State::Lost {
            death_exact_time: self.exact_time,
        };
        self.switch_time = FloatTime::ZERO;
        self.get_leaderboard(false);
    }

    pub fn get_leaderboard(&mut self, submit_score: bool) {
        self.transition = Some(Transition::LoadLeaderboard { submit_score });
    }
}
