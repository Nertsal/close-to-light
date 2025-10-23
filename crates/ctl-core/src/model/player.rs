use super::*;

/// Extra distance within which the player is still counted as in-light
/// to give some leeway on fading lights.
const LEEWAY: f32 = 0.025;

#[derive(Debug, Clone)]
pub struct Player {
    pub info: UserInfo,
    pub collider: Collider,
    pub health: Bounded<FloatTime>,

    /// Whether currently perfectly inside of any light.
    /// Controlled by the collider.
    pub is_perfect: bool,
    /// Lights which are at their waypoint and the player is perfectly inside.
    /// Controlled by the collider.
    pub perfect_waypoints: Vec<usize>,

    /// Event id of the closest friendly light.
    pub closest_light: Option<usize>,
    /// Distance to the closest friendly light.
    pub light_distance: Option<R32>,
    /// Distance to the closest dangerous light.
    pub danger_distance: Option<R32>,

    pub tail: Vec<PlayerTail>,
}

#[derive(Debug, Clone)]
pub struct PlayerTail {
    pub pos: vec2<Coord>,
    pub lifetime: Lifetime,
    pub state: LitState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LitState {
    Dark,
    Light { perfect: bool },
    Danger,
}

impl Player {
    pub fn new(collider: Collider, health: FloatTime) -> Self {
        Self {
            info: UserInfo {
                id: 0,
                name: "you".into(),
            },
            collider,
            health: Bounded::new_max(health),

            is_perfect: false,
            perfect_waypoints: Vec::new(),

            closest_light: None,
            light_distance: None,
            danger_distance: None,

            tail: Vec::new(),
        }
    }

    pub fn get_lit_state(&self) -> LitState {
        if self.danger_distance.is_some() {
            LitState::Danger
        } else if self.light_distance.is_some() {
            LitState::Light {
                perfect: self.is_perfect,
            }
        } else {
            LitState::Dark
        }
    }

    pub fn update_tail(&mut self, delta_time: FloatTime) {
        for tail in &mut self.tail {
            tail.lifetime.change(-delta_time);
        }
        self.tail.retain(|tail| tail.lifetime.is_above_min());

        let new_tail = PlayerTail {
            pos: self.collider.position,
            lifetime: Lifetime::new_max(r32(0.5)),
            state: self.get_lit_state(),
        };
        if let Some(last) = self.tail.last() {
            self.tail.push(PlayerTail {
                pos: (last.pos + new_tail.pos) / r32(2.0),
                ..new_tail
            });
        }
        self.tail.push(new_tail);
    }

    pub fn reset_distance(&mut self) {
        self.is_perfect = false;
        self.perfect_waypoints.clear();
        self.closest_light = None;
        self.light_distance = None;
        self.danger_distance = None;
    }

    pub fn update_distance_simple(&mut self, light: &Collider) {
        self.update_distance(light, None, false, R32::ZERO, false)
    }

    /// Update player's light distance, perfect measurement, and waypoint detection.
    /// Uses `last_rhythm` to account for completed rhythms to avoid double counting.
    pub fn update_light_distance(
        &mut self,
        light: &Light,
        last_rhythm: &HashMap<(usize, WaypointId), Time>,
    ) {
        let (time, waypoint) = light.closest_waypoint;
        let at_waypoint = time > -COYOTE_TIME
            && time < BUFFER_TIME
            && light
                .event_id
                .is_some_and(|event| !last_rhythm.contains_key(&(event, waypoint)));
        self.update_distance(
            &light.collider,
            light.event_id,
            light.danger,
            light.hollow,
            at_waypoint,
        )
    }

    fn update_distance(
        &mut self,
        light: &Collider,
        light_id: Option<usize>,
        danger: bool,
        hollow: R32,
        at_waypoint: bool,
    ) {
        let leeway = if danger {
            // NOTE: Danger lights do not give leeway (that would be the opposite of leeway)
            Coord::ZERO
        } else {
            Coord::new(LEEWAY)
        };
        let with_leeway = |distance: Coord| (distance - leeway).max(Coord::ZERO);

        let raw_distance = get_light_distance(self.collider.position, light, hollow);
        let min_distance = raw_distance.min;
        let max_distance = raw_distance.max;
        let raw_distance = with_leeway(raw_distance.raw);

        if !(min_distance..=max_distance).contains(&raw_distance) {
            // Outside of the light or inside of the hollow light
            return;
        }

        // Account for hollow lights
        let zero_distance = max_distance * (hollow.max(R32::ZERO) + r32(1.0)) / r32(2.0);
        let distance = (raw_distance - zero_distance).abs();

        let update = |value: &mut Option<Coord>| {
            *value = Some(value.map_or(distance, |value| value.min(distance)));
        };
        if danger {
            update(&mut self.danger_distance);
        } else {
            if self.light_distance.is_none_or(|old| distance < old) {
                self.light_distance = Some(distance);
                self.closest_light = light_id;
            }

            let radius = match self.collider.shape {
                Shape::Circle { radius } => radius,
                Shape::Line { .. } => unimplemented!(),
                Shape::Rectangle { .. } => unimplemented!(),
            };
            if distance < radius {
                self.is_perfect = true;
                if at_waypoint {
                    self.perfect_waypoints.extend(light_id);
                }
            }
        }
    }
}
