use super::*;

use geng_utils::conversions::Vec2RealConversions;

impl Model {
    pub fn update(&mut self, delta_time: Time) {
        self.beat_timer -= delta_time;
        while self.beat_timer < Time::ZERO {
            self.beat_timer += Time::ONE; // TODO bpm

            // Spawn a light at a random pos
            let mut rng = thread_rng();
            let position = vec2(rng.gen_range(-5.0..=5.0), rng.gen_range(-5.0..=5.0)).as_r32();
            let radius = r32(rng.gen_range(0.5..=1.0));
            self.lights.push(Light { position, radius });
        }
    }
}
