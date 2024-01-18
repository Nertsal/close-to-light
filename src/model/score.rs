use super::*;

const SCORE_VERSION: &str = "v0.1";

pub const DISCRETE_PERFECT: i32 = 1000;
pub const DISCRETE_OK: i32 = 100;
pub const DYNAMIC_SCALE: f32 = 1000.0;

#[derive(Debug, Clone)]
pub struct Score {
    pub calculated: CalculatedScore,
    pub metrics: ScoreMetrics,
}

/// Calculations based on the metrics.
#[derive(Debug, Clone)]
pub struct CalculatedScore {
    /// Combined score used as the main metric.
    pub combined: i32,
    pub accuracy: R32,
    pub precision: R32,
}

/// Raw metrics.
#[derive(Debug, Clone)]
pub struct ScoreMetrics {
    pub discrete: DiscreteMetrics,
    pub dynamic: DynamicMetrics,
}

/// Raw discrete metrics.
#[derive(Debug, Clone)]
pub struct DiscreteMetrics {
    /// The number of times player has been in `perfect` distance to a light.
    pub perfect: usize,
    pub total: usize,
    pub score: i32,
}

/// Raw dynamic/continuous metrics.
#[derive(Debug, Clone)]
pub struct DynamicMetrics {
    pub distance_sum: R32,
    pub frames: usize,
    pub score: i32,
}

impl Score {
    pub fn new() -> Self {
        Self {
            calculated: CalculatedScore::new(),
            metrics: ScoreMetrics::new(),
        }
    }

    /// Update the score given current player state.
    #[must_use]
    pub fn update(&mut self, player: &Player, delta_time: Time) -> Vec<GameEvent> {
        let events = self.metrics.update(player, delta_time);
        self.calculated = CalculatedScore::from_metrics(&self.metrics);
        events
    }
}

impl CalculatedScore {
    pub fn new() -> Self {
        Self {
            combined: 0,
            accuracy: R32::ONE,
            precision: R32::ONE,
        }
    }

    pub fn from_metrics(metrics: &ScoreMetrics) -> Self {
        let accuracy = if metrics.discrete.total == 0 {
            R32::ONE
        } else {
            r32(metrics.discrete.perfect as f32 / metrics.discrete.total as f32)
        };
        let precision =
            R32::ONE - metrics.dynamic.distance_sum / r32(metrics.dynamic.frames.max(1) as f32);

        Self {
            combined: metrics.discrete.score + metrics.dynamic.score,
            accuracy,
            precision,
        }
    }
}

impl ScoreMetrics {
    pub fn new() -> Self {
        Self {
            discrete: DiscreteMetrics::new(),
            dynamic: DynamicMetrics::new(),
        }
    }

    /// Update the metrics given the new player state.
    pub fn update(&mut self, player: &Player, delta_time: Time) -> Vec<GameEvent> {
        let mut events = Vec::new();
        if player.is_keyframe {
            events.extend(self.discrete.update(player));
        }
        events.extend(self.dynamic.update(player, delta_time));
        events
    }
}

impl DiscreteMetrics {
    pub fn new() -> Self {
        Self {
            perfect: 0,
            total: 0,
            score: 0,
        }
    }

    /// Update the metrics given the new player state.
    pub fn update(&mut self, player: &Player) -> Vec<GameEvent> {
        let mut events = Vec::new();

        if player.danger_distance.is_none() && player.light_distance.is_some() && player.is_perfect
        {
            self.perfect += 1;
            self.total += 1;
            self.score += DISCRETE_PERFECT;
            events.push(GameEvent::Rhythm { perfect: true });
        }

        events
    }

    pub fn missed_rhythm(&mut self) {
        self.total += 1;
        self.score += DISCRETE_OK;
    }
}

impl DynamicMetrics {
    pub fn new() -> Self {
        Self {
            distance_sum: R32::ZERO,
            frames: 0,
            score: 0,
        }
    }

    /// Update the metrics given the new player state.
    pub fn update(&mut self, player: &Player, delta_time: Time) -> Vec<GameEvent> {
        let events = Vec::new();

        self.frames += 1;
        self.distance_sum += player.light_distance.map_or(r32(1.0), |distance| {
            if player.is_perfect {
                r32(0.0)
            } else {
                distance.clamp(r32(0.0), r32(1.3)) / r32(1.3)
            }
        });

        if player.danger_distance.is_none() {
            if let Some(distance) = player.light_distance {
                let score_multiplier = (1.0 - distance.as_f32() + 0.5).min(1.0);
                self.score +=
                    (delta_time.as_f32() * score_multiplier * DYNAMIC_SCALE).ceil() as i32;
            }
        }

        events
    }
}
