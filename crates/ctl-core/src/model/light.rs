use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Light {
    /// Currently active collider.
    pub collider: Collider,
    /// The base collider used for reference.
    pub base_collider: Collider,
    /// Time since creation.
    pub lifetime: Time,
    /// Whether the light is dangerous.
    pub danger: bool,
    /// Makes the light hollow.
    pub hollow: R32,
    /// Id of the original event in the level.
    pub event_id: Option<usize>,
    /// Time delta to the closest waypoint.
    pub closest_waypoint: (Time, WaypointId),
}

#[derive(Debug, Clone)]
pub struct LightTelegraph {
    /// The light to telegraph.
    pub light: Light,
    /// Time since creation.
    pub lifetime: Time,
}

impl Light {
    pub fn into_telegraph(self) -> LightTelegraph {
        LightTelegraph {
            light: self,
            lifetime: Time::ZERO,
        }
    }

    pub fn contains_point(&self, position: vec2<Coord>) -> bool {
        let distance = get_light_distance(position, &self.collider, self.hollow);
        (distance.min..=distance.max).contains(&distance.raw)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LightDistance {
    pub raw: Coord,
    pub min: Coord,
    pub max: Coord,
}

pub fn get_light_distance(position: vec2<Coord>, light: &Collider, hollow: R32) -> LightDistance {
    let hollow = hollow.max(R32::ZERO);
    let delta_pos = position - light.position;
    match light.shape {
        Shape::Circle { radius } => LightDistance {
            raw: delta_pos.len(),
            min: hollow * radius,
            max: radius,
        },
        Shape::Line { width } => {
            let dir = light.rotation.unit_vec();
            let dir = vec2(-dir.y, dir.x); // perpendicular
            let dot = dir.x * delta_pos.x + dir.y * delta_pos.y;
            let radius = width / r32(2.0);
            LightDistance {
                raw: dot.abs(),
                min: hollow * radius,
                max: radius,
            }
        }
        Shape::Rectangle { width, height } => {
            let delta_pos = delta_pos.rotate(-light.rotation);
            let size = vec2(width, height);

            let mut angle = delta_pos.arg().normalized_pi() - Angle::from_degrees(r32(45.0));
            if angle.abs() > Angle::from_degrees(r32(90.0)) {
                angle -= Angle::from_degrees(r32(180.0) * angle.as_radians().signum());
            }
            let angle = angle + Angle::from_degrees(r32(45.0));

            let radius = if angle < size.arg().normalized_pi() {
                // On the right (vertical) side
                let h = vec2::dot(delta_pos, vec2::UNIT_Y);
                vec2(width / r32(2.0), h).len()
            } else {
                // On the top (horizontal) side
                let w = vec2::dot(delta_pos, vec2::UNIT_X);
                vec2(w, height / r32(2.0)).len()
            };
            LightDistance {
                raw: delta_pos.len().max(Coord::ZERO),
                min: hollow * radius,
                max: radius,
            }
        }
    }
}
