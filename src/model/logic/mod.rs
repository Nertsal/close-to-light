use super::*;

use geng_utils::conversions::Vec2RealConversions;

const BPM: f32 = 170.0;
const BPS: f32 = BPM / 60.0;

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
            self.beat_timer += r32(1.0 / BPS);

            // Spawn a light at a random pos
            let position = vec2(rng.gen_range(-5.0..=5.0), rng.gen_range(-5.0..=5.0)).as_r32();
            let rotation = Angle::from_degrees(r32(rng.gen_range(0.0..=360.0)));

            let shape_max = if rng.gen_bool(0.5) {
                Shape::Circle {
                    radius: r32(rng.gen_range(0.5..=1.0)),
                }
            } else {
                Shape::Line {
                    width: r32(rng.gen_range(0.2..=0.6)),
                }
            };

            self.lights.push(Light {
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
                lifetime: Lifetime::new_max(r32(1.5 / BPS)),
            });
        }

        // Update lights
        for light in &mut self.lights {
            light.lifetime.change(-delta_time);

            let t = 1.0 - light.lifetime.get_ratio().as_f32(); // 0 to 1
            let t = 1.0 - (t - 0.5).abs() * 2.0; // 0 to 1 to 0
            let t = 3.0 * t * t - 2.0 * t * t * t; // Smoothstep
                                                   // let t = t.sqrt(); // Square root for linear area change
            let t = r32(t);
            light.collider.shape = match light.shape_max {
                Shape::Circle { radius } => Shape::Circle { radius: radius * t },
                Shape::Line { width } => Shape::Line { width: width * t },
                Shape::Rectangle { width, height } => Shape::Rectangle {
                    width: width * t,
                    height: height * t,
                },
            };
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
}
