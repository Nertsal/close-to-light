use crate::{
    prelude::{HealthConfig, LevelModifiers, Score},
    task::Task,
    LeaderboardSecrets,
};

use geng::prelude::*;
use nertboard_client::{Nertboard, Player, ScoreEntry};

#[derive(Debug)]
pub enum LeaderboardStatus {
    None,
    Pending,
    Failed,
    Done,
}

pub struct Leaderboard {
    client: Option<Arc<Nertboard>>,
    task: Option<Task<anyhow::Result<Vec<ScoreEntry>>>>,
    pub status: LeaderboardStatus,
    pub loaded: LoadedBoard,
}

impl Clone for Leaderboard {
    fn clone(&self) -> Self {
        Self {
            client: self.client.as_ref().map(Arc::clone),
            task: None,
            status: LeaderboardStatus::None,
            loaded: LoadedBoard {
                category: self.loaded.category.clone(),
                local_high: self.loaded.local_high.clone(),
                ..LoadedBoard::new()
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedScore {
    pub player: String,
    pub score: i32,
    pub meta: ScoreMeta,
}

#[derive(Debug)]
pub struct LoadedBoard {
    pub category: ScoreCategory,
    pub my_position: Option<usize>,
    pub all_scores: Vec<ScoreEntry>,
    pub filtered: Vec<ScoreEntry>,
    pub local_high: Option<SavedScore>,
}

/// Meta information saved together with the score.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ScoreMeta {
    pub category: ScoreCategory,
    pub score: Score,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScoreCategory {
    version: u32,
    pub group: String,
    pub level: String,
    pub mods: LevelModifiers,
    pub health: HealthConfig,
}

impl Default for ScoreCategory {
    fn default() -> Self {
        Self::new(
            "".to_string(),
            "".to_string(),
            LevelModifiers::default(),
            HealthConfig::default(),
        )
    }
}

impl ScoreCategory {
    pub fn new(group: String, level: String, mods: LevelModifiers, health: HealthConfig) -> Self {
        Self {
            version: 2,
            group,
            level,
            mods,
            health,
        }
    }
}

impl ScoreMeta {
    pub fn new(
        group: String,
        level: String,
        mods: LevelModifiers,
        health: HealthConfig,
        score: Score,
    ) -> Self {
        Self {
            category: ScoreCategory::new(group, level, mods, health),
            score,
        }
    }
}

impl Leaderboard {
    pub fn new(secrets: Option<LeaderboardSecrets>) -> Self {
        let client = secrets.map(|secrets| {
            Arc::new(nertboard_client::Nertboard::new(secrets.url, Some(secrets.key)).unwrap())
        });
        Self {
            client,
            task: None,
            status: LeaderboardStatus::None,
            loaded: LoadedBoard::new(),
        }
    }

    /// The leaderboard needs to be polled to make progress.
    pub fn poll(&mut self) {
        if let Some(task) = &mut self.task {
            if let Some(res) = task.poll() {
                self.task = None;
                match res {
                    Ok(Ok(scores)) => {
                        log::debug!("Successfully loaded the leaderboard");
                        self.status = LeaderboardStatus::Done;
                        self.load_scores(scores);
                    }
                    Ok(Err(err)) | Err(err) => {
                        log::error!("Loading leaderboard failed: {:?}", err);
                        self.status = LeaderboardStatus::Failed;
                    }
                }
            }
        }
    }

    fn load_scores(&mut self, mut scores: Vec<ScoreEntry>) {
        scores.sort_by_key(|entry| -entry.score);
        self.loaded.all_scores = scores;
        self.loaded.refresh();
    }

    /// Change category filter using the cached scores if available.
    pub fn change_category(&mut self, category: ScoreCategory) {
        self.loaded.category = category;
        self.loaded.reload_local(None);
        match self.status {
            LeaderboardStatus::None | LeaderboardStatus::Failed => {
                self.refetch();
            }
            LeaderboardStatus::Pending => {}
            LeaderboardStatus::Done => {
                self.loaded.refresh();
            }
        }
    }

    /// Fetch scores from the server with the same meta.
    pub fn refetch(&mut self) {
        // Let the active task finish
        if self.task.is_some() {
            return;
        }

        if let Some(client) = &self.client {
            let board = Arc::clone(client);
            let future = async move {
                log::debug!("Fetching scores...");
                board.fetch_scores().await.map_err(anyhow::Error::from)
            };
            self.task = Some(Task::new(future));
            self.status = LeaderboardStatus::Pending;
        }
    }

    pub fn submit(&mut self, name: String, score: Option<i32>, meta: ScoreMeta) {
        let score = score.map(|score| SavedScore {
            player: name.clone(),
            score,
            meta: meta.clone(),
        });

        self.loaded.category = meta.category.clone();
        self.loaded.reload_local(score.as_ref());

        let meta_str = meta_str(&meta);
        let score = score.map(|score| ScoreEntry {
            player: score.player,
            score: score.score,
            extra_info: Some(meta_str),
        });

        if let Some(board) = &self.client {
            let board = Arc::clone(board);
            let future = async move {
                log::debug!("Submitting a score...");
                let name = name.as_str();
                let player =
                    if let Some(mut player) = preferences::load::<Player>(crate::PLAYER_STORAGE) {
                        log::debug!("Leaderboard: returning player");
                        if player.name == name {
                            // leaderboard.as_player(player.clone());
                            player
                        } else {
                            log::debug!("Leaderboard: name has changed");
                            player.name = name.to_owned();
                            preferences::save(crate::PLAYER_STORAGE, &player);
                            player.clone()
                        }
                    } else {
                        log::debug!("Leaderboard: new player");
                        let player = board.create_player(name).await.unwrap();
                        preferences::save(crate::PLAYER_STORAGE, &player);
                        player.clone()
                    };

                if let Some(score) = &score {
                    board.submit_score(&player, score).await.unwrap();
                }

                board.fetch_scores().await.map_err(anyhow::Error::from)
            };
            self.task = Some(Task::new(future));
            self.status = LeaderboardStatus::Pending;
        }
    }
}

impl LoadedBoard {
    fn new() -> Self {
        Self {
            category: ScoreCategory::new(
                "none".to_string(),
                "none".to_string(),
                LevelModifiers::default(),
                HealthConfig::default(),
            ),
            my_position: None,
            all_scores: Vec::new(),
            filtered: Vec::new(),
            local_high: None,
        }
    }

    pub fn reload_local(&mut self, score: Option<&SavedScore>) {
        log::debug!("Reloading local scores with a new score: {:?}", score);
        let mut highscores: Vec<SavedScore> =
            preferences::load(crate::HIGHSCORES_STORAGE).unwrap_or_default();
        let mut save = false;
        if let Some(highscore) = highscores
            .iter_mut()
            .find(|s| s.meta.category == self.category)
        {
            if let Some(score) = score {
                if score.score > highscore.score && score.meta.category == highscore.meta.category {
                    *highscore = score.clone();
                    save = true;
                }
            }
            self.local_high = Some(highscore.clone());
        } else if let Some(score) = score {
            highscores.push(score.clone());
            save = true;
            self.local_high = Some(score.clone());
        } else {
            self.local_high = None;
        }
        if save {
            preferences::save(crate::HIGHSCORES_STORAGE, &highscores);
        }
    }

    /// Refresh the filter.
    fn refresh(&mut self) {
        log::debug!("Filtering scores with meta\n{:#?}", self.category);

        let mut scores = self.all_scores.clone();

        // Filter for the same meta
        scores.retain(|entry| {
            !entry.player.is_empty()
                && entry.extra_info.as_ref().map_or(false, |info| {
                    serde_json::from_str::<ScoreMeta>(info)
                        .map_or(false, |entry_meta| entry_meta.category == self.category)
                })
        });

        {
            // TODO: unique players
            // Only leave unique names
            let mut i = 0;
            let mut names_seen = HashSet::new();
            while i < scores.len() {
                if !names_seen.contains(&scores[i].player) {
                    names_seen.insert(scores[i].player.clone());
                    i += 1;
                } else {
                    scores.remove(i);
                }
            }
        }

        self.filtered = scores;
        self.my_position = self.local_high.as_ref().and_then(|score| {
            self.filtered
                .iter()
                .position(|this| this.score == score.score)
        });
    }
}

fn meta_str(meta: &ScoreMeta) -> String {
    serde_json::to_string(meta).unwrap() // TODO: more compact?
}
