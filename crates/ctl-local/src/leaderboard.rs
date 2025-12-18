use ctl_client::Nertboard;
use ctl_core::{
    ScoreEntry, SubmitScore,
    auth::UserLogin,
    prelude::{HealthConfig, LevelModifiers, Score, Uuid},
    types::{Id, LevelInfo, UserInfo},
};
use ctl_util::Task;
use geng::prelude::*;

use crate::fs::LocalLevelId;

const SCORE_VERSION: u32 = 1;

#[derive(Debug)]
pub enum LeaderboardStatus {
    None,
    Pending,
    Failed,
    Done,
}

struct BoardUpdate {
    scores: Vec<ScoreEntry>,
}

#[derive(Clone)]
pub struct Leaderboard {
    inner: Rc<RefCell<LeaderboardImpl>>,
}

pub struct LeaderboardImpl {
    geng: Geng,
    fs: Rc<crate::fs::Controller>,
    /// Logged in as user with a name.
    pub user: Option<UserLogin>,
    pub client: Option<Arc<Nertboard>>,
    log_task: Option<Task<ctl_client::Result<Result<UserLogin, String>>>>,
    task: Option<Task<ctl_client::Result<BoardUpdate>>>,
    fs_task: Option<Task<anyhow::Result<Option<SavedScore>>>>,
    highscores_task: Option<Task<anyhow::Result<HashMap<LocalLevelId, SavedScore>>>>,
    pub status: LeaderboardStatus,
    pub loaded: LoadedBoard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedScore {
    pub user: UserInfo,
    pub score: i32,
    pub meta: ScoreMeta,
}

pub struct LoadedBoard {
    pub all_highscores: HashMap<LocalLevelId, SavedScore>,
    pub level: LevelInfo,
    pub player: Option<Id>,
    pub category: ScoreCategory,
    pub my_position: Option<usize>,
    pub all_scores: Vec<ScoreEntry>,
    pub filtered: Vec<ScoreEntry>,
    pub local_high: Option<SavedScore>,
}

/// Meta information saved together with the score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreMeta {
    pub category: ScoreCategory,
    pub score: Score,
    /// Number in range 0..=1 indicating level completion percentage.
    pub completion: R32,
    pub time: ::time::OffsetDateTime,
}

impl Default for ScoreMeta {
    fn default() -> Self {
        Self {
            category: ScoreCategory::default(),
            score: Score::default(),
            completion: R32::ZERO,
            time: ::time::OffsetDateTime::now_utc(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScoreCategory {
    version: u32,
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
        }
    }
}

impl Leaderboard {
    pub fn empty(geng: &Geng, fs: &Rc<crate::fs::Controller>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(LeaderboardImpl::empty(geng, fs))),
        }
    }

    pub fn new(
        geng: &Geng,
        client: Option<&Arc<Nertboard>>,
        fs: &Rc<crate::fs::Controller>,
    ) -> Self {
        Self {
            inner: Rc::new(RefCell::new(LeaderboardImpl::new(geng, client, fs))),
        }
    }

    pub fn get_mut(&mut self) -> RefMut<'_, LeaderboardImpl> {
        self.inner.borrow_mut()
    }

    pub fn get(&self) -> Ref<'_, LeaderboardImpl> {
        self.inner.borrow()
    }

    pub fn get_loaded(&self) -> Ref<'_, LoadedBoard> {
        Ref::map(self.inner.borrow(), |board| &board.loaded)
    }

    pub fn get_user(&self) -> Ref<'_, Option<UserLogin>> {
        Ref::map(self.inner.borrow(), |board| &board.user)
    }
}

impl LeaderboardImpl {
    pub fn empty(geng: &Geng, fs: &Rc<crate::fs::Controller>) -> Self {
        let mut leaderboard = Self {
            geng: geng.clone(),
            fs: fs.clone(),
            user: None,
            client: None,
            log_task: None,
            task: None,
            fs_task: None,
            highscores_task: None,
            status: LeaderboardStatus::None,
            loaded: LoadedBoard::new(),
        };
        leaderboard.refresh_local_highscores();
        leaderboard
    }

    pub fn new(
        geng: &Geng,
        client: Option<&Arc<Nertboard>>,
        fs: &Rc<crate::fs::Controller>,
    ) -> Self {
        let mut leaderboard = Self {
            client: client.cloned(),
            ..Self::empty(geng, fs)
        };
        leaderboard.relogin();
        leaderboard
    }

    /// Returns `true` if currently connecting to the server.
    pub fn is_connecting(&self) -> bool {
        self.log_task.is_some()
    }

    pub fn is_online(&self) -> bool {
        self.client
            .as_ref()
            .is_some_and(|client| client.is_online())
    }

    pub fn login_discord(&mut self) {
        if self.log_task.is_some() {
            return;
        }

        if let Some(client) = &self.client {
            let client = Arc::clone(client);
            let future = async move {
                let state = Uuid::new_v4().to_string();
                let redirect_uri = client.url.join("auth/discord")?;
                let url = format!(
                    "{}&state={}&redirect_uri={}",
                    ctl_core::DISCORD_LOGIN_URL,
                    state,
                    redirect_uri
                );
                if let Err(err) = webbrowser::open(&url) {
                    log::error!("failed to open login link: {err:?}");
                    return Err(ctl_client::ClientError::Connection);
                }
                client.login_external(state).await
            };
            self.log_task = Some(Task::new(&self.geng, future));
            self.user = None;
        }
    }

    /// Attempt to login back using the saved credentials.
    pub fn relogin(&mut self) {
        if self.log_task.is_some() {
            return;
        }

        if let Some(client) = &self.client {
            #[cfg(feature = "steam")]
            {
                // Login directly via Steam
                let client = Arc::clone(client);
                let future = async move { client.login_steam().await };
                self.log_task = Some(Task::new(&self.geng, future));
                self.user = None;
            }

            #[cfg(not(feature = "steam"))]
            {
                // Retrieve token
                let Some(user): Option<UserLogin> = preferences::load(crate::PLAYER_LOGIN_STORAGE)
                else {
                    return;
                };

                let client = Arc::clone(client);
                let future = async move { client.login_token(user.id, &user.token).await };
                self.log_task = Some(Task::new(&self.geng, future));
                self.user = None;
            }
        }
    }

    // pub fn login(&mut self, creds: Credentials) {
    //     if self.log_task.is_some() {
    //         return;
    //     }

    //     if let Some(client) = &self.client {
    //         let client = Arc::clone(client);
    //         let future = async move { client.login(&creds).await };
    //         self.log_task = Some(Task::new(&self.geng, future));
    //         self.user = None;
    //     }
    // }

    // pub fn register(&mut self, creds: Credentials) {
    //     if self.log_task.is_some() {
    //         return;
    //     }

    //     if let Some(client) = &self.client {
    //         let client = Arc::clone(client);
    //         let future = async move {
    //             if let Err(err) = client.register(&creds).await? {
    //                 return Ok(Err(err));
    //             }
    //             client.login(&creds).await
    //         };
    //         self.log_task = Some(Task::new(&self.geng, future));
    //         self.user = None;
    //     }
    // }

    pub fn logout(&mut self) {
        if self.log_task.is_some() {
            return;
        }

        if let Some(client) = &self.client {
            let client = Arc::clone(client);
            let token = self.user.as_ref().map(|user| user.token.clone());
            let future = async move {
                client.logout(token.as_deref()).await?;
                // TODO: log out is not an error
                Ok(Err("Logged out".to_string()))
            };
            self.log_task = Some(Task::new(&self.geng, future));
            self.user = None;
        }
    }

    /// The leaderboard needs to be polled to make progress.
    pub fn poll(&mut self) {
        if let Some(task) = self.log_task.take() {
            match task.poll() {
                Err(task) => self.log_task = Some(task),
                Ok(res) => {
                    match res {
                        Ok(Ok(user)) => {
                            log::debug!("Logged in as {}", &user.name);

                            #[cfg(not(feature = "steam"))] // Steam requires full relogin each time
                            preferences::save(crate::PLAYER_LOGIN_STORAGE, &user);

                            self.loaded.player = Some(user.id);
                            self.user = Some(user);
                        }
                        Ok(Err(err)) => {
                            if err == "Logged out" {
                                log::debug!("Logged out");
                                preferences::save(crate::PLAYER_LOGIN_STORAGE, &());
                            } else {
                                log::error!("Failed to log in: {err}");
                                // TODO: notification message
                            }
                        }
                        Err(err) => {
                            log::error!("Failed to log in: {err:?}");
                        }
                    }
                }
            }
        }

        if let Some(task) = self.task.take() {
            match task.poll() {
                Err(task) => self.task = Some(task),
                Ok(res) => match res {
                    Ok(update) => {
                        log::debug!("Successfully loaded the leaderboard");
                        self.status = LeaderboardStatus::Done;
                        self.load_scores(update.scores);
                    }
                    Err(err) => {
                        log::error!("Loading leaderboard failed: {err:?}");
                        self.status = LeaderboardStatus::Failed;
                    }
                },
            }
        }

        if let Some(task) = self.fs_task.take() {
            match task.poll() {
                Err(task) => self.fs_task = Some(task),
                Ok(res) => match res {
                    Ok(update) => {
                        log::debug!("Updating local highscore: {update:?}");
                        if let Some(score) = &update {
                            log::debug!("Adding to {:?} score {score:?}", self.loaded.level.hash);
                            self.loaded
                                .all_highscores
                                .insert(LocalLevelId::from_info(&self.loaded.level), score.clone());
                        }
                        self.loaded.local_high = update;
                    }
                    Err(err) => {
                        log::error!("Loading local scores failed: {err:?}");
                    }
                },
            }
        }

        if let Some(task) = self.highscores_task.take() {
            match task.poll() {
                Err(task) => self.highscores_task = Some(task),
                Ok(res) => match res {
                    Ok(update) => {
                        log::debug!("Loaded all local highscores");
                        self.loaded.all_highscores = update;
                    }
                    Err(err) => {
                        log::error!("Loading local highscores failed: {err:?}");
                    }
                },
            }
        }
    }

    fn refresh_local_highscores(&mut self) {
        if self.highscores_task.is_some() {
            return;
        }

        let fs = self.fs.clone();
        let future = async move { fs.load_local_highscores().await };
        self.highscores_task = Some(Task::new(&self.geng, future));
    }

    fn load_scores(&mut self, mut scores: Vec<ScoreEntry>) {
        scores.sort_by_key(|entry| -entry.score);
        self.loaded.all_scores = scores;
        self.loaded.refresh();
    }

    /// Change category filter using the cached scores if available.
    pub fn change_category(&mut self, category: ScoreCategory) {
        if self.loaded.category == category {
            return; // Unchanged
        }

        self.loaded.category = category;
        self.update_local(None);
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
            let level = self.loaded.level.id;
            let future = async move {
                log::debug!("Fetching scores for level {level}...");
                board
                    .fetch_scores(level)
                    .await
                    .map(|scores| BoardUpdate { scores })
            };
            self.task = Some(Task::new(&self.geng, future));
            self.status = LeaderboardStatus::Pending;
        }
    }

    pub fn reload_submit(
        &mut self,
        score: Option<i32>,
        submit_score: bool,
        level: LevelInfo,
        meta: ScoreMeta,
    ) {
        log::debug!("Reloading leaderboard for level {}", level.hash);
        let score = score.map(|score| SavedScore {
            user: self.user.as_ref().map_or(
                UserInfo {
                    id: 0,
                    name: "you".into(),
                },
                |user| UserInfo {
                    id: user.id,
                    name: user.name.clone(),
                },
            ),
            score,
            meta: meta.clone(),
        });

        self.loaded.level = level.clone();
        self.loaded.category = meta.category.clone();
        self.update_local(score.clone());

        if let Some(board) = &self.client {
            let mut score = score;
            if !submit_score || self.user.is_none() {
                score = None;
            }
            let board = Arc::clone(board);
            let level_hash = self.loaded.level.hash.clone();
            let future = async move {
                let meta_str = meta_str(&meta);
                let score = score.map(|score| SubmitScore {
                    score: score.score,
                    level_hash,
                    extra_info: Some(meta_str),
                });

                if let Some(score) = &score {
                    log::debug!("Submitting a score...");
                    board.submit_score(level.id, score).await?;
                }

                log::debug!("Fetching scores...");
                let scores = board.fetch_scores(level.id).await?;
                Ok(BoardUpdate { scores })
            };
            self.task = Some(Task::new(&self.geng, future));
            self.status = LeaderboardStatus::Pending;
        }
    }

    fn update_local(&mut self, score: Option<SavedScore>) {
        log::debug!("Updating local scores with a new score: {score:?}");
        let fs = self.fs.clone();
        let level_id = LocalLevelId::from_info(&self.loaded.level);
        let version = self.loaded.category.version;
        self.loaded.local_high = None;
        self.loaded.all_scores.clear();
        self.loaded.filtered.clear();
        let task = async move {
            let mut scores = match fs.load_local_scores(&level_id).await {
                Ok(scores) => scores,
                Err(err) => {
                    log::warn!("Loading local scores for level ({level_id:?}) failed: {err:?}");
                    vec![]
                }
            };
            if let Some(score) = score {
                scores.push(score);
                fs.save_local_scores(&level_id, &scores)
                    .await
                    .with_context(|| "when saving local scores")?;
            }
            let highscore = scores
                .iter()
                .filter(|score| score.meta.category.version == version)
                .max_by_key(|score| score.score)
                .cloned();
            Ok(highscore)
        };
        self.fs_task = Some(Task::new(&self.geng, task));
    }
}

impl LoadedBoard {
    fn new() -> Self {
        Self {
            all_highscores: HashMap::new(),
            level: LevelInfo::default(),
            player: None,
            category: ScoreCategory::new(LevelModifiers::default(), HealthConfig::default()),
            my_position: None,
            all_scores: Vec::new(),
            filtered: Vec::new(),
            local_high: None,
        }
    }

    /// Refresh the filter.
    fn refresh(&mut self) {
        log::debug!("Filtering scores with meta: {:?}", self.category);

        let mut scores = self.all_scores.clone();

        // Filter for the same meta
        scores.retain(|entry| {
            !entry.user.name.is_empty()
                && entry.extra_info.as_ref().is_some_and(|info| {
                    serde_json::from_str::<ScoreMeta>(info).is_ok_and(|entry_meta| {
                        entry_meta.category.version == self.category.version
                    })
                })
        });

        {
            // TODO: leave unique on server
            // Only leave unique players
            let mut i = 0;
            let mut ids_seen = HashSet::new();
            while i < scores.len() {
                if !ids_seen.contains(&scores[i].user.id) {
                    ids_seen.insert(scores[i].user.id);
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
