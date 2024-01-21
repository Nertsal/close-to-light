use crate::{
    prelude::{HealthConfig, Id, LevelModifiers},
    task::Task,
    LeaderboardSecrets,
};

use ctl_client::{
    core::{types::PlayerInfo, Player, ScoreEntry},
    Nertboard,
};
use geng::prelude::*;

#[derive(Debug)]
pub enum LeaderboardStatus {
    None,
    Pending,
    Failed,
    Done,
}

struct BoardUpdate {
    player: Option<Id>,
    scores: Vec<ScoreEntry>,
}

pub struct Leaderboard {
    client: Option<Arc<Nertboard>>,
    task: Option<Task<anyhow::Result<BoardUpdate>>>,
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
                meta: self.loaded.meta.clone(),
                local_high: self.loaded.local_high.clone(),
                ..LoadedBoard::new()
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedScore {
    pub level: Id,
    pub score: i32,
    pub meta: ScoreMeta,
}

#[derive(Debug)]
pub struct LoadedBoard {
    pub level: Id,
    pub meta: ScoreMeta,
    pub player: Option<Id>,
    pub my_position: Option<usize>,
    pub all_scores: Vec<ScoreEntry>,
    pub filtered: Vec<ScoreEntry>,
    pub local_high: Option<SavedScore>,
}

/// Meta information saved together with the score.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScoreMeta {
    pub version: u32,
    pub mods: LevelModifiers,
    pub health: HealthConfig,
}

impl ScoreMeta {
    pub fn new(mods: LevelModifiers, health: HealthConfig) -> Self {
        Self {
            version: 1,
            mods,
            health,
        }
    }
}

impl Leaderboard {
    pub fn new(secrets: Option<LeaderboardSecrets>) -> Self {
        let client = secrets.map(|secrets| {
            Arc::new(ctl_client::Nertboard::new(secrets.url, Some(secrets.key)).unwrap())
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
                    Ok(Ok(update)) => {
                        log::debug!("Successfully loaded the leaderboard");
                        self.status = LeaderboardStatus::Done;
                        self.loaded.player = update.player;
                        self.load_scores(update.scores);
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

    /// Change meta filter using the cached scores if available.
    pub fn change_meta(&mut self, meta: ScoreMeta) {
        self.loaded.meta = meta;
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
            let level = self.loaded.level;
            let future = async move {
                log::debug!("Fetching scores for level {}...", level);
                board
                    .fetch_scores(level)
                    .await
                    .map(|scores| BoardUpdate {
                        player: None,
                        scores,
                    })
                    .map_err(anyhow::Error::from)
            };
            self.task = Some(Task::new(future));
            self.status = LeaderboardStatus::Pending;
        }
    }

    pub fn submit(&mut self, name: String, score: Option<i32>, level: Id, meta: ScoreMeta) {
        let score = score.map(|score| SavedScore {
            level,
            score,
            meta: meta.clone(),
        });

        self.loaded.meta = meta.clone();
        self.loaded.reload_local(score.as_ref());

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

                let meta_str = meta_str(&meta);
                let score = score.map(|score| ScoreEntry {
                    player: PlayerInfo {
                        id: player.id,
                        name: name.to_owned(),
                    },
                    score: score.score,
                    extra_info: Some(meta_str),
                });

                if let Some(score) = &score {
                    board.submit_score(level, &player, score).await.unwrap();
                }

                let scores = board
                    .fetch_scores(level)
                    .await
                    .map_err(anyhow::Error::from)?;
                Ok(BoardUpdate {
                    player: Some(player.id),
                    scores,
                })
            };
            self.task = Some(Task::new(future));
            self.status = LeaderboardStatus::Pending;
        }
    }
}

impl LoadedBoard {
    fn new() -> Self {
        Self {
            level: 0,
            meta: ScoreMeta::new(LevelModifiers::default(), HealthConfig::default()),
            player: None,
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
        if let Some(highscore) = highscores.iter_mut().find(|s| s.meta == self.meta) {
            if let Some(score) = score {
                if score.score > highscore.score && score.meta == highscore.meta {
                    highscore.score = score.score;
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
        log::debug!("Filtering scores with meta\n{:#?}", self.meta);

        let mut scores = self.all_scores.clone();

        // Filter for the same meta
        scores.retain(|entry| {
            !entry.player.name.is_empty()
                && entry.extra_info.as_ref().map_or(false, |info| {
                    serde_json::from_str::<ScoreMeta>(info)
                        .map_or(false, |entry_meta| entry_meta == self.meta)
                })
        });

        {
            // TODO: leave unique on server
            // Only leave unique players
            let mut i = 0;
            let mut ids_seen = HashSet::new();
            while i < scores.len() {
                if !ids_seen.contains(&scores[i].player.id) {
                    ids_seen.insert(scores[i].player.id);
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
