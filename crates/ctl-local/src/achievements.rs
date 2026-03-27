use crate::{SavedScore, fs::LocalLevelId};

use std::collections::HashMap;

use ctl_core::model::ScoreGrade;
use geng::prelude::*;

/// ID's of the levels counted towards `LEVELS_COMPLETED`.
pub const ALL_DEMO_SONGS: [[u32; 3]; 4] = [[1, 2, 3], [4, 5, 6], [11, 12, 13], [14, 15, 16]];

#[cfg(feature = "demo")]
mod constants {
    pub const STAT_SONGS_TRIED: &str = "SONGS_TRIED";

    pub const STAT_LEVELS_COMPLETED: &str = "LEVELS_COMPLETED";
    pub const HARD_LEVEL_IDS: [u32; 4] = [3, 6, 13, 16];

    #[cfg(feature = "demo")]
    #[derive(Debug, Clone, Copy)]
    pub enum Achievement {
        /// Complete any level.
        FirstLights,
        /// Try every song.
        GettingStarted,
        /// Fail a hard level.
        AtTheEndOfTheTunnel,
        /// Complete a custom level.
        ExploratoryNature,
        /// Get grade A in any level.
        AForEffort,
        /// Complete every level.
        TotalIllumination,
    }

    impl Achievement {
        pub fn api_key(&self) -> &'static str {
            match self {
                Self::FirstLights => "COMPLETE_ANY_LEVEL",
                Self::GettingStarted => "COMPLETE_ALL_SONGS",
                Self::AtTheEndOfTheTunnel => "FAIL_HARD_LEVEL",
                Self::ExploratoryNature => "COMPLETE_CUSTOM_LEVEL",
                Self::AForEffort => "GET_GRADE_A",
                Self::TotalIllumination => "COMPLETE_ALL_LEVELS",
            }
        }
    }
}
#[cfg(not(feature = "demo"))]
mod constants {
    // TODO

    #[cfg(not(feature = "demo"))]
    #[derive(Debug)]
    pub enum Achievement {
        // TODO
    }

    impl Achievement {
        pub fn api_key(&self) -> &'static str {
            match self {
                _ => todo!(),
            }
        }
    }
}

use constants::*;

#[derive(Clone)]
pub struct Achievements {
    #[cfg(feature = "steam")]
    steam: Option<steamworks::Client>,
}

impl Default for Achievements {
    fn default() -> Self {
        Self::new()
    }
}

impl Achievements {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "steam")]
            steam: None,
        }
    }

    #[cfg(feature = "steam")]
    pub fn connect_steam(&mut self, steam: steamworks::Client) {
        steam.register_callback(|_: steamworks::UserStatsReceived| {});
        steam
            .user_stats()
            .request_user_stats(steam.user().steam_id().raw());
        self.steam = Some(steam);
    }

    /// Check if any achievements should be granted for the current highscores
    pub fn update_highscores(
        &self,
        highscores: &HashMap<LocalLevelId, SavedScore>,
        new_score: Option<(&LocalLevelId, &SavedScore)>,
    ) {
        #[cfg(feature = "demo")]
        {
            use ctl_core::types::Id;

            if let Some((new_score_level, new_score)) = new_score {
                // Check grade
                let new_grade = new_score.meta.calculate_grade();
                if new_grade >= ScoreGrade::A {
                    self.unlock_achievement(Achievement::AForEffort);
                }
                if new_grade == ScoreGrade::F
                    && let LocalLevelId::Id(id) = new_score_level
                    && HARD_LEVEL_IDS.contains(id)
                {
                    self.unlock_achievement(Achievement::AtTheEndOfTheTunnel)
                }

                // Any level completion
                if new_grade != ScoreGrade::F {
                    self.unlock_achievement(Achievement::FirstLights);

                    if let LocalLevelId::Id(id) = new_score_level
                        && !ALL_DEMO_SONGS.iter().any(|levels| levels.contains(id))
                    {
                        // Custom level
                        self.unlock_achievement(Achievement::ExploratoryNature);
                    }
                }
            }

            // Check completion progress
            let levels_completed: Vec<Id> = highscores
                .iter()
                .filter_map(|(id, score)| {
                    if let &LocalLevelId::Id(id) = id
                        && score.meta.calculate_grade() != ScoreGrade::F
                        && ALL_DEMO_SONGS.iter().any(|levels| levels.contains(&id))
                    {
                        Some(id)
                    } else {
                        None
                    }
                })
                .collect();
            if levels_completed.len() == ALL_DEMO_SONGS.iter().flatten().count() {
                self.unlock_achievement(Achievement::TotalIllumination);
            }

            // Song progress
            let songs_played = ALL_DEMO_SONGS
                .iter()
                .filter(|levels| levels.iter().any(|id| levels_completed.contains(id)))
                .count();
            if songs_played == ALL_DEMO_SONGS.len() {
                self.unlock_achievement(Achievement::GettingStarted);
            }

            // Stats tracking
            #[cfg(feature = "steam")]
            if let Some(steam) = &self.steam {
                let stats = steam.user_stats();
                log_steam_stat_error(
                    stats.set_stat_i32(STAT_LEVELS_COMPLETED, levels_completed.len() as i32),
                );
                log_steam_stat_error(stats.set_stat_i32(STAT_SONGS_TRIED, songs_played as i32));
                log_steam_stat_error(stats.store_stats());
            }
        }
        #[cfg(not(feature = "demo"))]
        {
            // TODO
            let _ = highscores;
            let _ = new_score;
        }
    }

    fn unlock_achievement(&self, achievement: Achievement) {
        log::info!("Unlocked achievement {:?}", achievement);

        #[cfg(feature = "steam")]
        if let Some(steam) = &self.steam
            && let Err(err) = steam.user_stats().achievement(achievement.api_key()).set()
        {
            log::error!(
                "Failed to set steam achievement {:?}: {:?}",
                achievement,
                err
            );
        }
    }
}

fn log_steam_stat_error(res: Result<(), ()>) {
    if let Err(()) = res {
        log::error!("Failed to update steam stats idk why");
    }
}
