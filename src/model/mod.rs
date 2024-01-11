mod collider;
mod level;
mod light;
mod logic;
mod movement;
mod options;
mod player;

pub use self::{collider::*, level::*, light::*, movement::*, options::*, player::*};

use crate::{game::PlayLevel, leaderboard::Leaderboard, prelude::*};

pub type Time = R32;
pub type Coord = R32;
pub type Lifetime = Bounded<Time>;
pub type Score = R32;

pub struct Music {
    pub meta: MusicMeta,
    sound: Rc<geng::Sound>,
    effect: Option<geng::SoundEffect>,
    volume: f64,
    /// Stop the music after the timer runs out.
    pub timer: Time,
}

impl Drop for Music {
    fn drop(&mut self) {
        self.stop();
    }
}

impl Debug for Music {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Music")
            .field("meta", &self.meta)
            // .field("effect", &self.effect)
            .field("volume", &self.volume)
            .field("timer", &self.timer)
            .finish()
    }
}

impl Clone for Music {
    fn clone(&self) -> Self {
        Self::new(self.sound.clone(), self.meta.clone())
    }
}

impl Music {
    pub fn new(sound: Rc<geng::Sound>, meta: MusicMeta) -> Self {
        Self {
            meta,
            sound,
            volume: 0.5,
            effect: None,
            timer: Time::ZERO,
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        let volume = f64::from(volume);
        let volume = volume.clamp(0.0, 1.0);
        self.volume = volume;
        if let Some(effect) = &mut self.effect {
            effect.set_volume(volume);
        }
    }

    pub fn stop(&mut self) {
        if let Some(mut effect) = self.effect.take() {
            effect.stop();
        }
        self.timer = Time::ZERO;
    }

    pub fn play_from(&mut self, time: time::Duration) {
        self.stop();
        let mut effect = self.sound.effect();
        effect.set_volume(self.volume);
        effect.play_from(time);
        self.effect = Some(effect);
    }

    pub fn beat_time(&self) -> Time {
        self.meta.beat_time()
    }
}

#[derive(Debug, Clone)]
pub struct HoverButton {
    pub base_collider: Collider,
    pub hover_time: Lifetime,
    pub animation: Movement,
}

impl HoverButton {
    pub fn new(collider: Collider, hover_time: impl Float) -> Self {
        Self {
            base_collider: collider,
            hover_time: Lifetime::new_zero(hover_time.as_r32()),
            animation: Movement {
                fade_in: r32(0.0),
                initial: Transform::scale(2.25),
                key_frames: vec![MoveFrame::scale(0.5, 5.0), MoveFrame::scale(0.25, 75.0)].into(),
                fade_out: r32(0.2),
            },
        }
    }

    /// Whether is button is now fading, i.e. going to finish its animation regardless of input.
    pub fn is_fading(&self) -> bool {
        // TODO: more custom
        self.hover_time.get_ratio().as_f32() > 0.5
    }

    pub fn update(&mut self, hovering: bool, delta_time: Time) {
        self.hover_time.change(if self.is_fading() || hovering {
            delta_time
        } else {
            -delta_time
        });
    }
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
    Exit,
}

pub struct Model {
    pub transition: Option<Transition>,
    pub assets: Rc<Assets>,
    pub leaderboard: Leaderboard,

    pub high_score: Score,
    pub camera: Camera2d,
    pub player: Player,

    pub options: Options,
    /// The level being played. Not changed, apart from music being played.
    pub level: PlayLevel,
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
    pub exit_button: HoverButton,
}

impl Model {
    pub fn new(
        assets: &Rc<Assets>,
        options: Options,
        level: PlayLevel,
        leaderboard: Leaderboard,
        player_name: String,
    ) -> Self {
        let start_time = level.start_time;
        let mut model = Self::empty(assets, options, level);
        model.player.name = player_name;
        model.leaderboard = leaderboard;

        model.init(start_time);
        model
    }

    pub fn empty(assets: &Rc<Assets>, options: Options, level: PlayLevel) -> Self {
        Self {
            leaderboard: Leaderboard::new(None),
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
                        radius: level.config.player.radius,
                    },
                ),
                level.config.health.max,
            ),
            restart_button: HoverButton::new(
                Collider::new(vec2(-3.0, 0.0).as_r32(), Shape::Circle { radius: r32(1.0) }),
                2.0,
            ),
            exit_button: HoverButton::new(
                Collider::new(vec2(-7.6, 3.7).as_r32(), Shape::Circle { radius: r32(0.6) }),
                3.0,
            ),
            options,
            level_state: LevelState::default(),
            level,
        }
    }
}
