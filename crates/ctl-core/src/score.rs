use crate::{
    model::{HealthConfig, LevelModifiers, Score, ScoreGrade},
    types::{FloatTime, Time, UserInfo},
};

use geng::prelude::*;

const SCORE_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitScore {
    pub level_hash: String,
    pub score: i32,
    pub meta: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerScore {
    pub user: UserInfo,
    pub score: i32,
    pub submitted_at: ::time::OffsetDateTime,
    pub meta: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreEntry {
    pub user: UserInfo,
    pub score: ScoreMeta,
}

/// Meta information saved together with the score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreMeta {
    pub category: ScoreCategory,
    pub score: Score,
    /// Number in range 0..=1 indicating level completion percentage.
    pub completion: R32,
    pub time: ::time::OffsetDateTime,
    #[serde(default)]
    pub pauses: Vec<PauseIndicator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PauseIndicator {
    /// Time of the level when the pause ocurred.
    pub time: Time,
    /// Duration of the pause in (real-time) seconds.
    pub duration: FloatTime,
}

impl Default for ScoreMeta {
    fn default() -> Self {
        Self {
            category: ScoreCategory::default(),
            score: Score::default(),
            completion: R32::ZERO,
            time: ::time::OffsetDateTime::now_utc(),
            pauses: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScoreCategory {
    pub version: u32,
    pub mods: LevelModifiers,
    pub health: HealthConfig,
}

impl Default for ScoreCategory {
    fn default() -> Self {
        Self::new(LevelModifiers::default(), HealthConfig::default())
    }
}

impl ScoreCategory {
    pub fn new(mods: LevelModifiers, health: HealthConfig) -> Self {
        Self {
            version: SCORE_VERSION,
            mods,
            health,
        }
    }
}

impl ScoreMeta {
    pub fn new(mods: LevelModifiers, health: HealthConfig, score: Score, completion: R32) -> Self {
        Self::new_category(ScoreCategory::new(mods, health), score, completion)
    }

    pub fn new_category(category: ScoreCategory, score: Score, completion: R32) -> Self {
        Self {
            category,
            score,
            completion,
            time: ::time::OffsetDateTime::now_utc(),
            pauses: Vec::new(),
        }
    }

    pub fn score(&self) -> i32 {
        self.score.calculated.combined
    }

    pub fn calculate_grade(&self) -> ScoreGrade {
        self.score.calculate_grade(self.completion)
    }
}
