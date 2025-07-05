use super::*;

pub const DISCRETE_PERFECT: i32 = 1000;
pub const DISCRETE_OK: i32 = 100;
pub const DYNAMIC_SCALE: f32 = 1000.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Score {
    pub multiplier: R32,
    pub calculated: CalculatedScore,
    pub metrics: ScoreMetrics,
}

/// Calculations based on the metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculatedScore {
    /// Combined score used as the main metric.
    pub combined: i32,
    pub accuracy: R32,
    pub precision: R32,
}

/// Raw metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreMetrics {
    pub discrete: DiscreteMetrics,
    pub dynamic: DynamicMetrics,
}

/// Raw discrete metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscreteMetrics {
    /// The number of times player has been in `perfect` distance to a light.
    pub perfect: usize,
    pub total: usize,
    pub score: i32,
}

/// Raw dynamic/continuous metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicMetrics {
    pub distance_sum: R32,
    pub frames: usize,
    pub score: i32,
}

impl Default for Score {
    fn default() -> Self {
        Self::new(R32::ONE)
    }
}

impl Score {
    pub fn new(multiplier: R32) -> Self {
        Self {
            multiplier,
            calculated: CalculatedScore::new(),
            metrics: ScoreMetrics::new(),
        }
    }

    /// Update the score given current player state.
    /// Returns `true` if the player hits the perfect rhythm.
    #[must_use]
    pub fn update(&mut self, player: &Player, delta_time: FloatTime) -> bool {
        let rhythm = self.metrics.update(player, delta_time);
        self.calculated = CalculatedScore::from_metrics(&self.metrics, self.multiplier);
        rhythm
    }
}

impl Default for CalculatedScore {
    fn default() -> Self {
        Self::new()
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

    pub fn from_metrics(metrics: &ScoreMetrics, multiplier: R32) -> Self {
        let accuracy = if metrics.discrete.total == 0 {
            R32::ONE
        } else {
            r32(metrics.discrete.perfect as f32 / metrics.discrete.total as f32)
        };
        let precision =
            R32::ONE - metrics.dynamic.distance_sum / r32(metrics.dynamic.frames.max(1) as f32);

        let discrete = (metrics.discrete.score as f32 * accuracy.as_f32()).ceil() as i32;

        Self {
            combined: ((discrete + metrics.dynamic.score) as f32 * multiplier.as_f32()) as i32,
            accuracy,
            precision,
        }
    }
}

impl Default for ScoreMetrics {
    fn default() -> Self {
        Self::new()
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
    pub fn update(&mut self, player: &Player, delta_time: FloatTime) -> bool {
        let mut rhythm = false;
        if player.is_keyframe {
            rhythm = rhythm || self.discrete.update(player);
        }
        self.dynamic.update(player, delta_time);
        rhythm
    }
}

impl Default for DiscreteMetrics {
    fn default() -> Self {
        Self::new()
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
    pub fn update(&mut self, player: &Player) -> bool {
        if player.danger_distance.is_none() && player.light_distance.is_some() && player.is_perfect
        {
            self.perfect += 1;
            self.total += 1;
            self.score += DISCRETE_PERFECT;
            true
        } else {
            false
        }
    }

    pub fn missed_rhythm(&mut self) {
        self.total += 1;
        self.score += DISCRETE_OK;
    }
}

impl Default for DynamicMetrics {
    fn default() -> Self {
        Self::new()
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
    pub fn update(&mut self, player: &Player, delta_time: FloatTime) {
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
    }
}
