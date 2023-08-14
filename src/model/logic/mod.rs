mod event;

use super::*;

use geng_utils::conversions::Vec2RealConversions;

impl Model {
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

        self.process_events(delta_time);

        // Update telegraphs
        for tele in &mut self.telegraphs {
            tele.lifetime.change(-delta_time);
            if tele.spawn_timer > Time::ZERO {
                tele.spawn_timer -= delta_time;
                if tele.spawn_timer <= Time::ZERO {
                    self.lights.push(tele.light.clone());
                }
            }

            let t = 1.0 - tele.lifetime.get_ratio().as_f32(); // 0 to 1
            let t = 3.0 * t * t - 2.0 * t * t * t; // Smoothstep
            tele.light.collider.shape = tele.light.shape_max.scaled(r32(t));
        }
        self.telegraphs.retain(|tele| tele.lifetime.is_above_min());

        // Update lights
        for light in &mut self.lights {
            light.lifetime.change(-delta_time);

            let t = 1.0 - light.lifetime.get_ratio().as_f32(); // 0 to 1
            let t = 1.0 - (t - 0.5).abs() * 2.0; // 0 to 1 to 0
            let t = 3.0 * t * t - 2.0 * t * t * t; // Smoothstep
            light.collider.shape = light.shape_max.scaled(r32(t));
        }
        self.lights.retain(|light| light.lifetime.is_above_min());

        // Check if the player is in light
        let lit = self
            .lights
            .iter()
            .any(|light| self.player.collider.check(&light.collider));
        if lit {
            self.player
                .fear_meter
                .change(-self.config.fear.restore_speed * delta_time);
        } else {
            self.player.fear_meter.change(delta_time);
        }
    }

    fn next_beat(&mut self) {
        self.current_beat += 1;

        if self.level.events.is_empty() {
            // No more events - start rng
            // TODO: end the level
            let telegraph = self.random_light_telegraphed();
            self.telegraphs.push(telegraph);
        } else {
            // Get the next events
            let mut to_remove = Vec::new();
            for (i, event) in self.level.events.iter().enumerate().rev() {
                if event.beat.floor().as_f32() as usize == self.current_beat {
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

    fn random_light_telegraphed(&self) -> LightTelegraph {
        self.random_light().into_telegraph(
            Telegraph {
                precede_time: r32(1.0),
                duration: r32(2.0),
            },
            self.level.beat_time(),
        )
    }

    fn random_light(&self) -> Light {
        let mut rng = thread_rng();

        let position = vec2(rng.gen_range(-5.0..=5.0), rng.gen_range(-5.0..=5.0)).as_r32();
        let rotation = Angle::from_degrees(r32(rng.gen_range(0.0..=360.0)));

        let shape_max = if rng.gen_bool(0.5) {
            Shape::Circle {
                radius: r32(rng.gen_range(0.5..=1.0)),
            }
        } else {
            Shape::Line {
                width: r32(rng.gen_range(0.5..=1.0)),
            }
        };

        Light {
            collider: {
                Collider {
                    position,
                    rotation,
                    shape: Shape::Circle {
                        radius: Coord::ZERO,
                    },
                }
            },
            shape_max,
            lifetime: Lifetime::new_max(r32(2.0) * self.level.beat_time()),
        }
    }
}
