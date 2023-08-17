mod event;

use super::*;

impl Model {
    /// Initialize the level by playing the events from the negative time.
    pub fn init(&mut self, target_time: Time) {
        log::info!("Replaying to the requested time {:.2}...", target_time);
        self.current_beat = self
            .level
            .events
            .iter()
            .map(|event| event.beat)
            .min()
            .map(|x| x.as_f32().floor() as isize - 1)
            .unwrap_or(0);

        let target_beat = target_time / self.level.beat_time();

        // Simulate at 60 fps
        let delta_time = Time::new(1.0 / 60.0);
        while (self.current_beat as f32) + 1.0 - self.beat_timer.as_f32() < target_beat.as_f32() {
            self.state = State::Playing;
            self.score = Score::ZERO;
            self.player.fear_meter.set_ratio(Time::ZERO);
            self.update(vec2::ZERO, delta_time);
        }
        log::info!("Replay finished");
    }

    pub fn update(&mut self, player_target: vec2<Coord>, delta_time: Time) {
        let mut rng = thread_rng();

        self.player.target_position = player_target;
        // Move
        self.player.collider.position = self.player.target_position;
        // Shake
        self.player.shake += Angle::from_degrees(r32(rng.gen_range(0.0..=360.0))).unit_vec()
            * self.config.fear.shake
            * self.player.fear_meter.get_ratio()
            * delta_time;
        self.player.shake = self.player.shake.clamp_len(..=r32(0.2));

        self.beat_timer -= delta_time;
        while self.beat_timer < Time::ZERO {
            self.beat_timer += self.level.beat_time();
            self.next_beat();
        }

        self.real_time += delta_time;
        self.switch_time += delta_time;

        self.process_events(delta_time);

        // Update telegraphs
        for tele in &mut self.telegraphs {
            tele.lifetime += delta_time;
            if tele.spawn_timer > Time::ZERO {
                tele.spawn_timer -= delta_time;
                if tele.spawn_timer <= Time::ZERO {
                    self.lights.push(tele.light.clone());
                    continue;
                }
            }

            let t = tele.lifetime * tele.speed;
            let transform = if t > tele.light.movement.duration() {
                Transform {
                    scale: Coord::ZERO,
                    ..default()
                }
            } else {
                tele.light.movement.get(t)
            };
            tele.light.collider = tele.light.base_collider.transformed(transform);
        }
        self.telegraphs.retain(|tele| {
            tele.spawn_timer > Time::ZERO
                || tele.lifetime * tele.speed < tele.light.movement.duration()
        });

        // Update lights
        for light in &mut self.lights {
            light.lifetime += delta_time;

            let transform = light.movement.get(light.lifetime);
            light.collider = light.base_collider.transformed(transform);
        }
        self.lights
            .retain(|light| light.lifetime < light.movement.duration());

        if let State::Playing = self.state {
            // Check if the player is in light
            self.player.is_in_light = self
                .lights
                .iter()
                .any(|light| self.player.collider.check(&light.collider));
            if self.player.is_in_light {
                self.player
                    .fear_meter
                    .change(-self.config.fear.restore_speed * delta_time);
                self.score += delta_time * r32(10.0);
            } else {
                self.player.fear_meter.change(delta_time);
            }

            if self.player.fear_meter.is_max() {
                self.state = State::Lost;
                self.music.stop();
                self.switch_time = Time::ZERO;
            }
        } else if self.switch_time > Time::ONE {
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
    }

    fn restart(&mut self) {
        log::info!("Restarting...");
        *self = Self::new(
            &self.assets,
            self.config.clone(),
            self.level_clone.clone(),
            Time::ZERO,
        );
    }

    fn next_beat(&mut self) {
        self.current_beat += 1;

        if let State::Playing = self.state {
            if self.level.events.is_empty() {
                if self.level.rng_end {
                    // No more events - start rng
                    let telegraph = self.random_light_telegraphed();
                    self.telegraphs.push(telegraph);
                } else if self.lights.is_empty() {
                    self.state = State::Finished;
                    self.music.stop();
                    self.switch_time = Time::ZERO;
                }
            } else {
                // Get the next events
                let mut to_remove = Vec::new();
                for (i, event) in self.level.events.iter().enumerate().rev() {
                    if event.beat.floor().as_f32() as isize == self.current_beat {
                        to_remove.push(i);
                    }
                }
                for i in to_remove {
                    let event = self.level.events.swap_remove(i);
                    self.queued_events.push(QueuedEvent {
                        delay: event.beat.fract() * self.level.beat_time(),
                        event: event.event,
                    });
                }
            }
        }
    }

    fn random_light_telegraphed(&self) -> LightTelegraph {
        self.random_light().into_telegraph(
            Telegraph {
                precede_time: r32(1.0),
                speed: r32(1.0),
            },
            self.level.beat_time(),
        )
    }

    fn random_light(&self) -> Light {
        let mut rng = thread_rng();

        let position = vec2(rng.gen_range(-5.0..=5.0), rng.gen_range(-5.0..=5.0)).as_r32();
        let rotation = Angle::from_degrees(r32(rng.gen_range(0.0..=360.0)));

        let shape = *self
            .config
            .shapes
            .choose(&mut rng)
            .expect("no shapes available");

        let collider = Collider {
            position,
            rotation,
            shape,
        };
        Light {
            base_collider: collider.clone(),
            collider,
            movement: Movement {
                key_frames: vec![
                    MoveFrame {
                        lerp_time: 0.0,
                        transform: Transform {
                            scale: r32(0.0),
                            ..default()
                        },
                    },
                    MoveFrame {
                        lerp_time: 1.0,
                        transform: Transform::identity(),
                    },
                    MoveFrame {
                        lerp_time: 1.0,
                        transform: Transform {
                            scale: r32(0.0),
                            ..default()
                        },
                    },
                ]
                .into(),
            }
            .with_beat_time(self.level.beat_time()),
            lifetime: Time::ZERO,
            // lifetime: Lifetime::new_max(r32(2.0) * self.level.beat_time()),
        }
    }
}
