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
    pub fn update_fire(&mut self, delta_time: FloatTime) {
        // Move fire
        for particle in &mut self.fire {
            particle.position += vec2(0.0, 2.0).as_r32() * delta_time;
            particle.size -= r32(0.5) * delta_time;
        }
        self.fire.retain(|particle| particle.size.as_f32() > 0.0);

        // Spawn more fire
        let mut rng = thread_rng();
        for light in &self.level_state.lights {
            if !light.fire {
                continue;
            }

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
                .extend(pos.into_iter().map(|position| FireParticle {
                    position,
                    size,
                    danger: light.danger,
                }));
        }
    }
}
