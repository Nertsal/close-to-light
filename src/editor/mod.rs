mod config;
mod handle_event;
mod ui;

pub use self::{config::*, ui::*};

use crate::{
    prelude::*,
    render::editor::{EditorRender, RenderOptions},
};

#[derive(Debug, Clone)]
pub enum State {
    Idle,
    /// Place a new light.
    Place {
        shape: Shape,
        danger: bool,
    },
    /// Specify a movement path for the light.
    Movement {
        start_beat: Time,
        light: LightEvent,
        redo_stack: Vec<MoveFrame>,
    },
    Playing {
        start_beat: Time,
        old_state: Box<State>,
    },
}

#[derive(Default)]
pub struct EditorLevelState {
    /// Interactable level state representing current time.
    pub static_level: Option<LevelState>,
    /// Dynamic level state showing the upcoming animations.
    pub dynamic_level: Option<LevelState>,
    /// Index of the hovered static light.
    pub hovered_light: Option<usize>,
}

impl EditorLevelState {
    pub fn relevant(&self) -> &LevelState {
        self.static_level
            .as_ref()
            .or(self.dynamic_level.as_ref())
            .expect("level editor has no displayable state")
    }

    /// Returns the index of the hovered event (if any).
    pub fn hovered_event(&self) -> Option<usize> {
        self.hovered_light.and_then(|i| self.light_event(i))
    }

    pub fn light_event(&self, light: usize) -> Option<usize> {
        self.static_level
            .as_ref()
            .and_then(|level| level.lights.get(light))
            .and_then(|light| light.event_id)
    }
}

pub struct EditorState {
    geng: Geng,
    assets: Rc<Assets>,
    transition: Option<geng::state::Transition>,
    render: EditorRender,
    editor: Editor,
    framebuffer_size: vec2<usize>,
    cursor_pos: vec2<f64>,
    render_options: RenderOptions,
    ui: EditorUI,
}

pub struct Editor {
    pub config: EditorConfig,
    pub cursor_world_pos: vec2<Coord>,
    pub level_path: std::path::PathBuf,
    pub level: Level,
    /// Simulation model.
    pub model: Model,
    pub level_state: EditorLevelState,
    pub grid_size: Coord,
    pub current_beat: Time,
    pub real_time: Time,
    pub selected_light: Option<usize>,
    /// At what rotation the objects should be placed.
    pub place_rotation: Angle<Coord>,
    pub state: State,
    pub music: geng::SoundEffect,
    /// Stop the music after the timer runs out.
    pub music_timer: Time,
    /// Whether the last frame was scrolled through time.
    pub was_scrolling: bool,
    /// Whether currently scrolling through time.
    /// Used as a hack to not replay the music every frame.
    pub scrolling: bool,
    pub snap_to_grid: bool,
    /// Whether to visualize the lights' movement for the current beat.
    pub visualize_beat: bool,
}

impl EditorState {
    pub fn new(
        geng: Geng,
        assets: Rc<Assets>,
        config: EditorConfig,
        game_config: Config,
        level: Level,
        level_path: std::path::PathBuf,
    ) -> Self {
        let model = Model::empty(&assets, game_config, level.clone());
        Self {
            transition: None,
            render: EditorRender::new(&geng, &assets),
            framebuffer_size: vec2(1, 1),
            cursor_pos: vec2::ZERO,
            render_options: RenderOptions {
                show_grid: true,
                hide_ui: false,
            },
            ui: EditorUI::new(),
            editor: Editor {
                grid_size: Coord::new(model.camera.fov) / config.grid.height,
                cursor_world_pos: vec2::ZERO,
                level_state: EditorLevelState::default(),
                current_beat: Time::ZERO,
                real_time: Time::ZERO,
                selected_light: None,
                place_rotation: Angle::ZERO,
                state: State::Idle,
                music: assets.music.effect(),
                music_timer: Time::ZERO,
                was_scrolling: false,
                scrolling: false,
                visualize_beat: true,
                snap_to_grid: true,
                config,
                model,
                level_path,
                level,
            },
            geng,
            assets,
        }
    }

    fn scroll_time(&mut self, delta: Time) {
        self.editor.current_beat = (self.editor.current_beat + delta).max(Time::ZERO);
        self.editor.scrolling = true;
    }

    fn snap_pos_grid(&self, pos: vec2<Coord>) -> vec2<Coord> {
        (pos / self.editor.grid_size).map(Coord::round) * self.editor.grid_size
    }

    /// Start playing the game from the current time.
    fn play_game(&mut self) {
        self.transition = Some(geng::state::Transition::Push(Box::new(
            crate::game::Game::new(
                &self.geng,
                &self.assets,
                self.editor.model.config.clone(),
                self.editor.level.clone(),
                None,
                String::new(),
                self.editor.current_beat * self.editor.level.beat_time(),
            ),
        )));
    }

    fn undo(&mut self) {
        match &mut self.editor.state {
            State::Playing { .. } => {}
            State::Movement {
                light, redo_stack, ..
            } => {
                let frames = &mut light.light.movement.key_frames;
                // Skip the fade in frames
                if frames.len() > 2 {
                    let frame = frames.pop_back().unwrap();
                    redo_stack.push(frame);
                }
            }
            State::Place { .. } => {
                // TODO: idk
            }
            State::Idle => {
                // TODO: remove last added sequence or restore last removed
            }
        }
    }

    fn redo(&mut self) {
        match &mut self.editor.state {
            State::Playing { .. } => {}
            State::Movement {
                light, redo_stack, ..
            } => {
                if let Some(frame) = redo_stack.pop() {
                    light.light.movement.key_frames.push_back(frame);
                }
            }
            State::Place { .. } => {
                // TODO
            }
            State::Idle => {
                // TODO
            }
        }
    }

    fn save(&mut self) {
        let result = (|| -> anyhow::Result<()> {
            // TODO: switch back to ron
            // https://github.com/geng-engine/geng/issues/71
            let level = serde_json::to_string_pretty(&self.editor.level)?;
            let mut writer =
                std::io::BufWriter::new(std::fs::File::create(&self.editor.level_path)?);
            write!(writer, "{}", level)?;
            Ok(())
        })();
        match result {
            Ok(()) => {
                self.editor.model.level = self.editor.level.clone();
            }
            Err(err) => {
                log::error!("Failed to save the level: {:?}", err);
            }
        }
    }
}

impl geng::State for EditorState {
    fn transition(&mut self) -> Option<geng::state::Transition> {
        self.transition.take()
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as f32);
        self.editor.real_time += delta_time;

        if self.editor.music_timer > Time::ZERO {
            self.editor.music_timer -= delta_time;
            if self.editor.music_timer <= Time::ZERO {
                self.editor.music.stop();
            }
        }

        if self.editor.scrolling {
            self.editor.was_scrolling = true;
        } else {
            if self.editor.was_scrolling {
                // Stopped scrolling
                // Play some music
                self.editor.music.stop();
                self.editor.music = self.assets.music.effect();
                self.editor.music.play_from(time::Duration::from_secs_f64(
                    (self.editor.current_beat * self.editor.level.beat_time()).as_f32() as f64,
                ));
                self.editor.music_timer =
                    self.editor.level.beat_time() * self.editor.config.playback_duration;
            }
            self.editor.was_scrolling = false;
        }

        self.editor.scrolling = false;

        if let State::Playing { .. } = self.editor.state {
            self.editor.current_beat = self.editor.real_time / self.editor.level.beat_time();
        }

        let pos = self.cursor_pos.as_f32();
        let pos = pos - self.ui.game.bottom_left();
        let pos = self
            .editor
            .model
            .camera
            .screen_to_world(self.ui.game.size(), pos)
            .as_r32();
        self.editor.cursor_world_pos = if self.editor.snap_to_grid {
            self.snap_pos_grid(pos)
        } else {
            pos
        };

        self.editor.render_lights(self.editor.visualize_beat);
    }

    fn handle_event(&mut self, event: geng::Event) {
        self.handle_event(event);
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        self.ui = EditorUI::layout(
            &self.editor,
            Aabb2::ZERO.extend_positive(framebuffer.size().as_f32()),
            self.cursor_pos.as_f32(),
        );
        self.render
            .draw_editor(&self.editor, &self.ui, &self.render_options, framebuffer);
    }
}

impl Editor {
    pub fn render_lights(&mut self, visualize_beat: bool) {
        let (static_time, dynamic_time) = if let State::Playing { .. } = self.state {
            // TODO: self.music.play_position()
            (None, Some(self.current_beat))
        } else {
            let time = self.current_beat;
            let dynamic = if visualize_beat {
                Some(time + (self.real_time / self.level.beat_time()).fract())
            } else {
                None
            };
            (Some(time), dynamic)
        };

        let mut static_level = static_time.map(|time| LevelState::render(&self.level, time, None));
        let mut dynamic_level =
            dynamic_time.map(|time| LevelState::render(&self.level, time, None));

        if let State::Movement {
            start_beat, light, ..
        } = &self.state
        {
            for level in [&mut static_level, &mut dynamic_level]
                .into_iter()
                .flatten()
            {
                level.render_event(&commit_light(*start_beat, light.clone()), None);
            }
        }

        let mut hovered_light = None;
        if let Some(level) = &static_level {
            for (i, light) in level.lights.iter().enumerate() {
                if light.collider.contains(self.cursor_world_pos) {
                    hovered_light = Some(i);
                }
            }
        }

        self.level_state = EditorLevelState {
            static_level,
            dynamic_level,
            hovered_light,
        };
    }
}

fn commit_light(start_beat: Time, mut light: LightEvent) -> TimedEvent {
    // Add fade out
    light.light.movement.key_frames.push_back(MoveFrame {
        lerp_time: Time::ONE, // in beats
        transform: Transform {
            scale: Coord::ZERO,
            ..default()
        },
    });

    // Commit event
    TimedEvent {
        beat: start_beat,
        event: Event::Light(light),
    }
}
