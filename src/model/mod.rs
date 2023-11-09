mod collider;
mod config;
mod level;
mod light;
mod logic;
mod movement;
mod player;

pub use self::{collider::*, config::*, level::*, light::*, movement::*, player::*};

use crate::{leaderboard::Leaderboard, prelude::*, LeaderboardSecrets};

pub type Time = R32;
pub type Coord = R32;
pub type Lifetime = Bounded<Time>;
pub type Score = R32;

#[derive(Debug, Clone)]
pub struct HoverButton {
    pub collider: Collider,
    pub hover_time: Lifetime,
}

#[derive(Debug, Clone)]
pub enum State {
    /// Wait for the player to hover the light and some additional time.
    Starting {
        /// Time until we can start the game.
        start_timer: Time,
        /// Time to start playing music from.
        music_start_time: Time,
    },
    Playing,
    Lost {
        /// The time of death.
        death_beat_time: Time,
    },
    Finished,
}

pub enum Transition {
    LoadLeaderboard { submit_score: bool },
}

pub enum LeaderboardState {
    None,
    Pending,
    Ready(Leaderboard),
}

pub struct Model {
    pub transition: Option<Transition>,
    pub assets: Rc<Assets>,
    pub config: Config,
    pub secrets: Option<LeaderboardSecrets>,
    pub leaderboard: LeaderboardState,
    pub music: Option<geng::SoundEffect>,

    pub high_score: Score,
    pub camera: Camera2d,
    pub player: Player,

    /// The level being played. Not changed.
    pub level: Level,
    /// Current state of the level.
    pub level_state: LevelState,
    pub state: State,
    pub score: Score,

    pub real_time: Time,
    /// Time since the last state change.
    pub switch_time: Time,
    /// Current time with beats as measure.
    pub beat_time: Time,

    // for Lost/Finished state
    pub restart_button: HoverButton,
}

impl Drop for Model {
    fn drop(&mut self) {
        self.stop_music();
    }
}

impl Model {
    pub fn new(
        assets: &Rc<Assets>,
        config: Config,
        level: Level,
        leaderboard: Option<LeaderboardSecrets>,
        player_name: String,
        start_time: Time,
    ) -> Self {
        let mut model = Self::empty(assets, config, level);
        model.secrets = leaderboard;
        model.player.name = player_name;

        model.init(start_time);
        model
    }

    pub fn empty(assets: &Rc<Assets>, config: Config, level: Level) -> Self {
        Self {
            transition: None,
            assets: assets.clone(),
            state: State::Starting {
                start_timer: Time::ZERO, // reset during init
                music_start_time: Time::ZERO,
            },
            score: Score::ZERO,
            high_score: preferences::load("highscore").unwrap_or(Score::ZERO),
            beat_time: Time::ZERO,
            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 10.0,
            },
            real_time: Time::ZERO,
            switch_time: Time::ZERO,
            player: Player::new(
                Collider::new(
                    vec2::ZERO,
                    Shape::Circle {
                        radius: r32(config.player.radius),
                    },
                ),
                level.config.health.max,
            ),
            restart_button: HoverButton {
                collider: Collider::new(
                    vec2(-3.0, 0.0).as_r32(),
                    Shape::Circle { radius: r32(1.0) },
                ),
                hover_time: Lifetime::new(Time::ZERO, Time::ZERO..=r32(3.0)),
            },
            config,
            secrets: None,
            leaderboard: LeaderboardState::None,
            level_state: LevelState::default(),
            level,
            music: None,
        }
    }

    fn stop_music(&mut self) {
        if let Some(mut music) = self.music.take() {
            music.stop();
        }
    }
}
