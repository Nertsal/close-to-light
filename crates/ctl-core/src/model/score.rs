use super::*;

pub const DISCRETE_PERFECT: i32 = 1000;
pub const DISCRETE_OK: i32 = 100;
pub const DYNAMIC_SCALE: f32 = 1000.0;
/// The maximum distance where precision matters, beyond that distance
/// everything is disregarded as too far from the light.
pub const MAX_PREC_DISTANCE: f32 = 1.5;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScoreGrade {
    F,
    D,
    C,
    B,
    A,
    S,
    SS,
    SSS,
}

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
    /// Total sum of distances to the center of the closest light source each frame.
    pub distance_sum: R32,
    /// Total number of simulation frames.
    pub frames: usize,
    /// Total score awarded for being close to light.
    pub score: i32,
    /// Total number of frames when the player is perfectly centered on a light.
    pub frames_perfect: usize,
    /// Total number of frames when the player is inside of a light (perfect or not).
    pub frames_light: usize,
    /// Total number of frames when the player is not touching any light.
    pub frames_black: usize,
    /// Total number of frames when the player is inside of a red light.
    pub frames_red: usize,
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

    pub fn calculate_grade(&self, completion: R32) -> ScoreGrade {
        // TODO: change 0.999 to 1.0 (only affects old clients)
        if completion.as_f32() < 0.999 {
            return ScoreGrade::F;
        }
        let acc = self.calculated.accuracy.as_f32();
        if acc >= 1.0 {
            if self.calculated.precision.as_f32() >= 0.95 {
                ScoreGrade::SSS
            } else {
                ScoreGrade::SS
            }
        } else if acc >= 0.95 {
            ScoreGrade::S
        } else if acc >= 0.9 {
            ScoreGrade::A
        } else if acc >= 0.75 {
            ScoreGrade::B
        } else if acc >= 0.5 {
            ScoreGrade::C
        } else {
            ScoreGrade::D
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
        let rhythm = self.discrete.update(player);
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
        if player.danger_distance.is_none() && !player.perfect_waypoints.is_empty() {
            self.perfect += player.perfect_waypoints.len();
            self.total += player.perfect_waypoints.len();
            self.score += DISCRETE_PERFECT * player.perfect_waypoints.len() as i32;
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
            frames_perfect: 0,
            frames_light: 0,
            frames_black: 0,
            frames_red: 0,
        }
    }

    /// Update the metrics given the new player state.
    pub fn update(&mut self, player: &Player, delta_time: FloatTime) {
        self.frames += 1;
        self.distance_sum += player.light_distance.map_or(r32(1.0), |distance| {
            if player.is_perfect {
                r32(0.0)
            } else {
                distance.clamp(r32(0.0), r32(MAX_PREC_DISTANCE)) / r32(MAX_PREC_DISTANCE)
            }
        });

        if player.danger_distance.is_none() {
            if let Some(distance) = player.light_distance {
                self.frames_light += 1;
                let d = if player.is_perfect {
                    self.frames_perfect += 1;
                    0.0
                } else {
                    distance.as_f32()
                };
                let score_multiplier = (1.0 - d + 0.5).min(1.0);
                self.score +=
                    (delta_time.as_f32() * score_multiplier * DYNAMIC_SCALE).ceil() as i32;
            } else {
                self.frames_black += 1;
            }
        } else {
            self.frames_red += 1;
        }
    }
}
