use super::*;

#[derive(Debug, Clone)]
pub struct Player {
    pub name: String,
    pub shake: vec2<Coord>,
    pub collider: Collider,
    pub health: Bounded<Time>,
    // pub is_in_light: bool,
    /// Normalized distance to the closest friendly light.
    pub light_distance_normalized: Option<R32>,
    /// Normalized distance to the closest dangerous light.
    pub danger_distance_normalized: Option<R32>,
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
    pub fn new(collider: Collider, health: Time) -> Self {
        Self {
            name: "anonymous".to_string(),
            shake: vec2::ZERO,
            collider,
            health: Bounded::new_max(health),
            light_distance_normalized: None,
            danger_distance_normalized: None,
            tail: Vec::new(),
        }
    }

    pub fn update_tail(&mut self, delta_time: Time) {
        for tail in &mut self.tail {
            tail.lifetime.change(-delta_time);
        }
        self.tail.retain(|tail| tail.lifetime.is_above_min());
        let new_tail = PlayerTail {
            pos: self.collider.position,
            lifetime: Lifetime::new_max(r32(0.5)),
            state: if self.danger_distance_normalized.is_some() {
                LitState::Danger
            } else if self.light_distance_normalized.is_some() {
                LitState::Light
            } else {
                LitState::Dark
            },
        };
        if let Some(last) = self.tail.last() {
            self.tail.push(PlayerTail {
                pos: (last.pos + new_tail.pos) / r32(2.0),
                ..new_tail
            });
        }
        self.tail.push(new_tail);
    }
}
