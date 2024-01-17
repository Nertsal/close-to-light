use super::*;

const SCORE_VERSION: &str = "v0.1";

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
    /// The number of times player has been in `perfect` distance to a light.
    pub total: usize,
}

/// Raw dynamic/continuous metrics.
#[derive(Debug, Clone)]
pub struct DynamicMetrics {
    pub distance_sum: R32,
    pub frames: usize,
}

impl Score {
    pub fn new() -> Self {
        Self {
            calculated: CalculatedScore::new(),
            metrics: ScoreMetrics::new(),
        }
    }

    /// Update the score given current player state.
    pub fn update(&mut self, player: &Player, delta_time: Time) {
        self.metrics.update(player, delta_time);
        self.calculated = CalculatedScore::from_metrics(&self.metrics);
    }
}

impl CalculatedScore {
    pub fn new() -> Self {
        Self {
            combined: 0,
            accuracy: R32::ZERO,
            precision: R32::ZERO,
        }
    }

    pub fn from_metrics(metrics: &ScoreMetrics) -> Self {
        Self {
            combined: 0,
            accuracy: r32(metrics.discrete.perfect as f32 / metrics.discrete.total.max(1) as f32),
            precision: R32::ONE
                - metrics.dynamic.distance_sum / r32(metrics.dynamic.frames.max(1) as f32),
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
    pub fn update(&mut self, player: &Player, delta_time: Time) {
        if player.is_keyframe {
            self.discrete.update(player);
        }
        self.dynamic.update(player, delta_time);
    }
}

impl DiscreteMetrics {
    pub fn new() -> Self {
        Self {
            perfect: 0,
            total: 0,
        }
    }

    /// Update the metrics given the new player state.
    pub fn update(&mut self, player: &Player) {
        if player.danger_distance.is_none() && player.light_distance.is_some() {
            if player.is_perfect {
                self.perfect += 1;
            }
            self.total += 1;
        }
    }
}

impl DynamicMetrics {
    pub fn new() -> Self {
        Self {
            distance_sum: R32::ZERO,
            frames: 0,
        }
    }

    /// Update the metrics given the new player state.
    pub fn update(&mut self, player: &Player, _delta_time: Time) {
        self.frames += 1;
        self.distance_sum += player.light_distance.map_or(r32(1.0), |distance| {
            if player.is_perfect {
                r32(0.0)
            } else {
                distance.clamp(r32(0.0), r32(1.3)) / r32(1.3)
            }
        });
    }
}

// let score_multiplier = (r32(1.0) - distance + r32(0.5)).min(r32(1.0));
// self.score += delta_time * score_multiplier * r32(100.0);
