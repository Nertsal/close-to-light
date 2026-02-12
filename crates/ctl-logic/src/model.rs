use super::*;

use ctl_core::score::PauseIndicator;
use ctl_local::{CachedGroup, Leaderboard, LocalMusic};
use generational_arena::Index;

#[derive(Debug, Clone)]
pub struct PlayGroup {
    pub group_index: Index,
    pub cached: Rc<CachedGroup>,
    pub music: Option<Rc<LocalMusic>>,
}

#[derive(Debug, Clone)]
pub struct PlayLevel {
    pub group: PlayGroup,
    pub level_index: usize,
    pub level: LevelFull,
    pub config: LevelConfig,
    pub start_time: Time,
    pub transition_button: Option<HoverButton>,
}

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
                initial: WaypointInitial::new(seconds_to_time(0.5), TransformLight::scale(2.25)),
                waypoints: vec![Waypoint::scale(seconds_to_time(0.25), 5.0)].into(),
                last: TransformLight::scale(75.0),
            },
            clicked: false,
        }
    }

    /// Returns the collider that should be currently active,
    /// based on the current hover_time and the set animation.
    pub fn get_relevant_collider(&self) -> Collider {
        let t = self.hover_time.get_ratio();
        let scale = self.animation.get(seconds_to_time(t)).scale;
        self.base_collider
            .transformed(TransformLight { scale, ..default() })
    }

    /// Whether is button is now fading, i.e. going to finish its animation regardless of input.
    pub fn is_fading(&self) -> bool {
        // TODO: more custom
        self.hover_time.get_ratio().as_f32() > 0.6
    }

    pub fn reset(&mut self) {
        self.clicked = false;
        self.hover_time.set_ratio(FloatTime::ZERO);
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
        death_time_ms: Time,
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

    pub camera: Camera2d,
    pub player: Player,
    /// Whether the cursor clicked last frame.
    pub cursor_clicked: bool,
    pub vfx: Vfx,

    pub music_offset: Time,
    /// The level being played. Not changed, apart from music being played.
    pub level: PlayLevel,
    /// Current state of the level.
    pub level_state: LevelState,
    pub state: State,
    pub score: Score,
    /// Button that was used to transition into the game.
    pub transition_button: Option<HoverButton>,

    /// List collected rhythm (event_id, waypoint_id)
    /// with times of their collection.
    pub recent_rhythm: HashMap<(usize, WaypointId), Time>,
    /// Waypoint rhythms tracking player accuracy.
    pub rhythms: Vec<Rhythm>,
    /// Times when the game was paused.
    pub pauses: Vec<PauseIndicator>,

    /// Real time that has passed since the level was opened.
    pub real_time: FloatTime,
    /// Time since the last state change.
    pub switch_time: FloatTime,
    /// Time since the level was started playing.
    pub play_time: FloatTime,
    /// Current exact play time in milliseconds.
    pub play_time_ms: Time,
    /// Time since the level started playing, counting towards level completion.
    pub completion_time: FloatTime,
    /// Restart and Exit buttons timer.
    pub button_time: FloatTime,
    /// Whether Restart and Exit buttons are activated.
    pub buttons_active: bool,

    // for Lost/Finished state
    pub restart_button: HoverButton,
    pub exit_button: HoverButton,
}

impl Model {
    pub fn new(context: Context, level: PlayLevel, leaderboard: Leaderboard) -> Self {
        let start_time = level.start_time;
        let mut model = Self::empty(context, level);
        if let Some(player) = &*leaderboard.get_user() {
            model.player.info = UserInfo {
                id: player.id,
                name: player.name.clone(),
            };
        }
        model.leaderboard = leaderboard;

        model.init(start_time);
        model
    }

    pub fn empty(context: Context, level: PlayLevel) -> Self {
        context.music.stop();
        let options = context.get_options();
        Self {
            transition: None,
            leaderboard: Leaderboard::empty(
                &context.geng,
                &context.local.fs,
                &context.achievements,
            ),
            context,

            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: Camera2dFov::Cover {
                    width: 17.778,
                    height: 10.0,
                    scale: 1.0,
                },
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
            vfx: Vfx::new(),

            music_offset: seconds_to_time(r32(options.gameplay.music_offset * 1e-3)),
            level_state: LevelState::default(),
            state: State::Starting {
                start_timer: FloatTime::ZERO, // reset during init
                music_start_time: Time::ZERO,
            },
            score: Score::new(level.config.modifiers.multiplier()),

            recent_rhythm: HashMap::new(),
            rhythms: Vec::new(),
            pauses: Vec::new(),

            real_time: FloatTime::ZERO,
            switch_time: FloatTime::ZERO,
            play_time: FloatTime::ZERO,
            play_time_ms: Time::ZERO,
            completion_time: FloatTime::ZERO,
            button_time: FloatTime::ZERO,
            buttons_active: false,

            restart_button: HoverButton::new(
                Collider::new(vec2(-3.0, 0.0).as_r32(), Shape::Circle { radius: r32(1.0) }),
                2.0,
            ),
            exit_button: HoverButton::new(
                Collider::new(vec2(-7.6, 3.7).as_r32(), Shape::Circle { radius: r32(0.6) }),
                3.0,
            ),

            transition_button: level.transition_button.clone(),
            level,
        }
    }
}
