mod level;
mod logic;
mod options;
mod player;
mod score;

pub use self::{level::*, options::*, player::*, score::*};

use crate::{game::PlayLevel, leaderboard::Leaderboard, prelude::*};

const COYOTE_TIME: Time = TIME_IN_FLOAT_TIME / 10; // 0.1s
const BUFFER_TIME: Time = TIME_IN_FLOAT_TIME / 10; // 0.1s

pub type Lifetime = Bounded<FloatTime>;

#[derive(Debug, Clone)]
pub enum GameEvent {
    Rhythm { perfect: bool },
}

#[derive(Debug, Clone)]
pub struct HoverButton {
    pub base_collider: Collider,
    pub hover_time: Lifetime,
    pub animation: Movement,
    pub clicked: bool,
}

impl HoverButton {
    pub fn new(collider: Collider, hover_time: impl Float) -> Self {
        Self {
            base_collider: collider,
            hover_time: Lifetime::new_zero(hover_time.as_r32()),
            animation: Movement {
                fade_in: seconds_to_time(r32(0.0)),
                initial: Transform::scale(2.25),
                key_frames: vec![MoveFrame::scale(0.2, 5.0), MoveFrame::scale(0.1, 75.0)].into(),
                interpolation: MoveInterpolation::Smoothstep,
                curve: TrajectoryInterpolation::Linear,
                fade_out: seconds_to_time(r32(0.1)),
            },
            clicked: false,
        }
    }

    /// Whether is button is now fading, i.e. going to finish its animation regardless of input.
    pub fn is_fading(&self) -> bool {
        // TODO: more custom
        self.hover_time.get_ratio().as_f32() > 0.6
    }

    pub fn update(&mut self, hovering: bool, delta_time: FloatTime) {
        let scale = if self.is_fading() {
            self.clicked = false;
            1.0
        } else if self.clicked {
            3.0
        } else if hovering {
            1.0
        } else {
            -1.0
        };
        let dt = r32(scale) * delta_time;
        self.hover_time.change(dt);
    }
}

#[derive(Debug, Clone)]
pub enum State {
    /// Wait for the player to hover the light and some additional time.
    Starting {
        /// Time until we can start the game.
        start_timer: FloatTime,
        /// Time to start playing music from.
        music_start_time: Time,
    },
    Playing,
    Lost {
        /// The time of death.
        death_exact_time: Time,
    },
    Finished,
}

pub enum Transition {
    LoadLeaderboard { submit_score: bool },
    Exit,
}

#[derive(Debug, Clone)]
pub struct Rhythm {
    /// Position where the rhythm occured.
    pub position: vec2<Coord>,
    /// Time since the beat.
    pub time: Bounded<Time>,
    /// Whether player input was perfect at the beat.
    pub perfect: bool,
}

pub struct Model {
    pub context: Context,
    pub transition: Option<Transition>,
    pub leaderboard: Leaderboard,

    pub high_score: i32,
    pub camera: Camera2d,
    pub player: Player,
    /// Whether the cursor clicked last frame.
    pub cursor_clicked: bool,

    pub options: Options,
    /// The level being played. Not changed, apart from music being played.
    pub level: PlayLevel,
    /// Current state of the level.
    pub level_state: LevelState,
    pub state: State,
    pub score: Score,

    /// List collected rhythm (event_id, waypoint_id).
    pub last_rhythm: (usize, WaypointId),
    /// Waypoint rhythms.
    pub rhythms: Vec<Rhythm>,

    pub real_time: FloatTime,
    /// Time since the last state change.
    pub switch_time: FloatTime,
    /// Current exact time in milliseconds.
    pub exact_time: Time,

    // for Lost/Finished state
    pub restart_button: HoverButton,
    pub exit_button: HoverButton,
}

impl Model {
    pub fn new(
        context: Context,
        options: Options,
        level: PlayLevel,
        mut leaderboard: Leaderboard,
    ) -> Self {
        leaderboard.loaded.level = level.level.meta.id;

        let start_time = level.start_time;
        let mut model = Self::empty(context, options, level);
        if let Some(player) = &leaderboard.user {
            model.player.info = UserInfo {
                id: player.id,
                name: player.name.clone(),
            };
        }
        model.leaderboard = leaderboard;

        model.init(start_time);
        model
    }

    pub fn empty(context: Context, options: Options, level: PlayLevel) -> Self {
        Self {
            transition: None,
            leaderboard: Leaderboard::empty(&context.geng),
            context,

            high_score: preferences::load("highscore").unwrap_or(0), // TODO: save score version
            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 10.0,
            },
            player: Player::new(
                Collider::new(
                    vec2::ZERO,
                    Shape::Circle {
                        radius: level.config.player.radius,
                    },
                ),
                level.config.health.max,
            ),
            cursor_clicked: false,

            level_state: LevelState::default(),
            state: State::Starting {
                start_timer: FloatTime::ZERO, // reset during init
                music_start_time: Time::ZERO,
            },
            score: Score::new(level.config.modifiers.multiplier()),

            last_rhythm: (999, WaypointId::Frame(999)), // Should be never the first one
            rhythms: Vec::new(),

            exact_time: Time::ZERO,
            real_time: FloatTime::ZERO,
            switch_time: FloatTime::ZERO,

            restart_button: HoverButton::new(
                Collider::new(vec2(-3.0, 0.0).as_r32(), Shape::Circle { radius: r32(1.0) }),
                2.0,
            ),
            exit_button: HoverButton::new(
                Collider::new(vec2(-7.6, 3.7).as_r32(), Shape::Circle { radius: r32(0.6) }),
                3.0,
            ),

            options,
            level,
        }
    }
}
