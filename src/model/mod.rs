mod collider;
mod config;
mod level;
mod light;
mod logic;
mod movement;

pub use self::{collider::*, config::*, level::*, light::*, movement::*};

use crate::{assets::Assets, leaderboard::Leaderboard, LeaderboardSecrets};

use std::collections::VecDeque;

use geng::prelude::*;
use geng_utils::{bounded::Bounded, conversions::Vec2RealConversions};

pub type Time = R32;
pub type Coord = R32;
pub type Lifetime = Bounded<Time>;
pub type Score = R32;

#[derive(Debug, Clone)]
pub struct HoverButton {
    pub collider: Collider,
    pub hover_time: Lifetime,
}

#[derive(Debug)]
pub struct QueuedEvent {
    /// Delay until the event should happen (in seconds).
    pub delay: Time,
    pub event: Event,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub name: String,
    pub target_position: vec2<Coord>,
    pub shake: vec2<Coord>,
    pub collider: Collider,
    pub fear_meter: Bounded<Time>,
    // pub is_in_light: bool,
    pub light_distance_normalized: Option<R32>,
}

impl Player {
    pub fn is_in_light(&self) -> bool {
        self.light_distance_normalized.is_some()
    }
}

#[derive(Debug, Clone)]
pub enum State {
    Playing,
    Lost,
    Finished,
}

pub struct Model {
    pub assets: Rc<Assets>,
    pub config: Config,
    pub secrets: Option<LeaderboardSecrets>,
    pub leaderboard: Option<Leaderboard>,
    /// The level to use when restarting the game.
    pub level_clone: Level,
    pub level: Level,
    pub music: geng::SoundEffect,
    pub state: State,
    pub score: Score,
    pub high_score: Score,
    /// Can be negative when initializing (because of simulating negative time).
    pub current_beat: isize,
    pub camera: Camera2d,
    pub real_time: Time,
    /// Time since the last state change.
    pub switch_time: Time,
    /// The time until the next music beat.
    pub beat_timer: Time,
    pub queued_events: Vec<QueuedEvent>,
    pub player: Player,
    pub telegraphs: Vec<LightTelegraph>,
    pub lights: Vec<Light>,

    // for Lost/Finished state
    pub restart_button: HoverButton,
}

impl Drop for Model {
    fn drop(&mut self) {
        self.music.stop();
    }
}

impl Model {
    pub fn new(
        assets: &Rc<Assets>,
        config: Config,
        level: Level,
        leaderboard: Option<LeaderboardSecrets>,
        start_time: Time,
    ) -> Self {
        let mut model = Self {
            assets: assets.clone(),
            state: State::Playing,
            score: Score::ZERO,
            high_score: preferences::load("highscore").unwrap_or(Score::ZERO),
            current_beat: 0,
            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 10.0,
            },
            real_time: Time::ZERO,
            switch_time: Time::ZERO,
            beat_timer: Time::ZERO,
            queued_events: Vec::new(),
            player: Player {
                name: "".to_string(),
                target_position: vec2::ZERO,
                shake: vec2::ZERO,
                collider: Collider::new(
                    vec2::ZERO,
                    Shape::Circle {
                        radius: r32(config.player.radius),
                    },
                ),
                fear_meter: Bounded::new(r32(0.0), r32(0.0)..=r32(1.0)),
                light_distance_normalized: None,
            },
            telegraphs: vec![],
            lights: vec![],
            restart_button: HoverButton {
                collider: Collider::new(
                    vec2(-3.0, 0.0).as_r32(),
                    Shape::Circle { radius: r32(1.0) },
                ),
                hover_time: Lifetime::new(Time::ZERO, Time::ZERO..=r32(3.0)),
            },
            config,
            secrets: leaderboard,
            leaderboard: None,
            level_clone: level.clone(),
            level,
            music: assets.music.effect(),
        };
        model.init(start_time);
        model
            .music
            .play_from(time::Duration::from_secs_f64(start_time.as_f32() as f64));
        model
    }
}
