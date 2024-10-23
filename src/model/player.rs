use super::*;

#[derive(Debug, Clone)]
pub struct Player {
    pub info: UserInfo,
    pub shake: vec2<Coord>,
    pub collider: Collider,
    pub health: Bounded<FloatTime>,

    /// Whether currently perfectly inside the center of the light.
    /// Controlled by the collider.
    pub is_perfect: bool,
    /// Whether currently closest light is in a keyframe.
    pub is_keyframe: bool,

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
    Light,
    Danger,
}

impl Player {
    pub fn new(collider: Collider, health: FloatTime) -> Self {
        Self {
            info: UserInfo {
                id: 0,
                name: "you".into(),
            },
            shake: vec2::ZERO,
            collider,
            health: Bounded::new_max(health),

            is_perfect: false,
            is_keyframe: false,

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
            LitState::Light
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
        self.is_keyframe = false;
        self.closest_light = None;
        self.light_distance = None;
        self.danger_distance = None;
    }

    pub fn update_distance_simple(&mut self, light: &Collider) {
        self.update_distance(light, None, false, false)
    }

    pub fn update_distance(
        &mut self,
        light: &Collider,
        light_id: Option<usize>,
        danger: bool,
        at_waypoint: bool,
    ) {
        let delta_pos = self.collider.position - light.position;
        let (raw_distance, max_distance) = match light.shape {
            Shape::Circle { radius } => (delta_pos.len(), radius),
            Shape::Line { width } => {
                let dir = light.rotation.unit_vec();
                let dir = vec2(-dir.y, dir.x); // perpendicular
                let dot = dir.x * delta_pos.x + dir.y * delta_pos.y;
                (dot.abs(), width / r32(2.0))
            }
            Shape::Rectangle { .. } => todo!(),
        };

        if raw_distance > max_distance {
            return;
        }

        let update = |value: &mut Option<Coord>| {
            *value = Some(value.map_or(raw_distance, |value| value.min(raw_distance)));
        };
        if danger {
            update(&mut self.danger_distance);
        } else {
            if self.light_distance.map_or(true, |old| raw_distance < old) {
                self.light_distance = Some(raw_distance);
                self.closest_light = light_id;
                self.is_keyframe = at_waypoint;
            }

            let radius = match self.collider.shape {
                Shape::Circle { radius } => radius,
                Shape::Line { .. } => unimplemented!(),
                Shape::Rectangle { .. } => unimplemented!(),
            };
            self.is_perfect = raw_distance < radius;
        }
    }

    pub fn update_light_distance(&mut self, light: &Light, last_rhythm: (usize, WaypointId)) {
        let (time, waypoint) = light.closest_waypoint;
        let at_waypoint = time > -COYOTE_TIME
            && time < BUFFER_TIME
            && light
                .event_id
                .map_or(false, |event| last_rhythm != (event, waypoint));
        self.update_distance(&light.collider, light.event_id, light.danger, at_waypoint)
    }
}
