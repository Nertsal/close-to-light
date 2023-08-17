mod config;
mod draw;
mod handle_event;

pub use self::config::*;

use crate::{assets::*, model::*, render::UtilRender};

use geng::prelude::*;
use geng_utils::conversions::Vec2RealConversions;

#[derive(Debug, Clone)]
enum State {
    /// Place a new light.
    Place,
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

pub struct Editor {
    geng: Geng,
    assets: Rc<Assets>,
    config: EditorConfig,
    util_render: UtilRender,
    pixel_texture: ugli::Texture,
    ui_texture: ugli::Texture,
    framebuffer_size: vec2<usize>,
    cursor_pos: vec2<f64>,
    cursor_world_pos: vec2<Coord>,
    level: Level,
    /// Simulation model.
    model: Model,
    /// Lights (with transparency and hover) ready for visualization and hover detection.
    rendered_lights: Vec<(Light, f32, bool)>,
    /// Telegraphs (with transparency and hover) ready for visualization.
    rendered_telegraphs: Vec<(LightTelegraph, f32, bool)>,
    /// Index of the hovered light in the `level.events`.
    hovered_light: Option<usize>,
    current_beat: Time,
    time: Time,
    /// Whether to visualize the lights' movement for the current beat.
    visualize_beat: bool,
    selected_shape: usize,
    /// At what rotation the objects should be placed.
    place_rotation: Angle<Coord>,
    state: State,
    music: geng::SoundEffect,
    /// Stop the music after the timer runs out.
    music_timer: Time,
    /// Whether the last frame was scrolled through time.
    was_scrolling: bool,
    /// Whether currently scrolling through time.
    /// Used as a hack to not replay the music every frame.
    scrolling: bool,
    grid_size: Coord,
    show_grid: bool,
    snap_to_grid: bool,
}

impl Editor {
    pub fn new(
        geng: Geng,
        assets: Rc<Assets>,
        config: EditorConfig,
        game_config: Config,
        level: Level,
    ) -> Self {
        let mut pixel_texture =
            geng_utils::texture::new_texture(geng.ugli(), vec2(360 * 16 / 9, 360));
        pixel_texture.set_filter(ugli::Filter::Nearest);
        let mut ui_texture =
            geng_utils::texture::new_texture(geng.ugli(), vec2(1080 * 16 / 9, 1080));
        ui_texture.set_filter(ugli::Filter::Nearest);

        let model = Model::new(game_config, level.clone());
        Self {
            util_render: UtilRender::new(&geng, &assets),
            pixel_texture,
            ui_texture,
            framebuffer_size: vec2(1, 1),
            cursor_pos: vec2::ZERO,
            cursor_world_pos: vec2::ZERO,
            rendered_lights: vec![],
            rendered_telegraphs: vec![],
            hovered_light: None,
            current_beat: Time::ZERO,
            time: Time::ZERO,
            visualize_beat: true,
            selected_shape: 0,
            place_rotation: Angle::ZERO,
            state: State::Place,
            music: assets.music.effect(),
            music_timer: Time::ZERO,
            was_scrolling: false,
            scrolling: false,
            grid_size: Coord::new(model.camera.fov) / config.grid.height,
            show_grid: true,
            snap_to_grid: true,
            model,
            geng,
            assets,
            config,
            level,
        }
    }

    fn scroll_time(&mut self, delta: Time) {
        self.current_beat = (self.current_beat + delta).max(Time::ZERO);
        self.scrolling = true;
    }

    fn snap_pos_grid(&self, pos: vec2<Coord>) -> vec2<Coord> {
        (pos / self.grid_size).map(Coord::round) * self.grid_size
    }

    fn undo(&mut self) {
        match &mut self.state {
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
            State::Place => {
                // TODO: remove last added sequence or restore last removed
            }
        }
    }

    fn redo(&mut self) {
        match &mut self.state {
            State::Playing { .. } => {}
            State::Movement {
                light, redo_stack, ..
            } => {
                if let Some(frame) = redo_stack.pop() {
                    light.light.movement.key_frames.push_back(frame);
                }
            }
            State::Place => {
                // TODO
            }
        }
    }

    fn save(&mut self) {
        let result = (|| -> anyhow::Result<()> {
            // TODO: switch back to ron
            // https://github.com/geng-engine/geng/issues/71
            let level = serde_json::to_string_pretty(&self.level)?;
            let path = run_dir().join("assets").join("level.json");
            let mut writer = std::io::BufWriter::new(std::fs::File::create(path)?);
            write!(writer, "{}", level)?;
            Ok(())
        })();
        match result {
            Ok(()) => {
                self.model.level = self.level.clone();
            }
            Err(err) => {
                log::error!("Failed to save the level: {:?}", err);
            }
        }
    }
}

impl geng::State for Editor {
    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as f32);
        self.time += delta_time;

        if self.music_timer > Time::ZERO {
            self.music_timer -= delta_time;
            if self.music_timer <= Time::ZERO {
                self.music.stop();
            }
        }

        if self.scrolling {
            self.was_scrolling = true;
        } else {
            if self.was_scrolling {
                // Stopped scrolling
                // Play some music
                self.music.stop();
                self.music = self.assets.music.effect();
                self.music.play_from(time::Duration::from_secs_f64(
                    (self.current_beat * self.level.beat_time()).as_f32() as f64,
                ));
                self.music_timer = self.level.beat_time() * self.config.playback_duration;
            }
            self.was_scrolling = false;
        }

        self.scrolling = false;

        if let State::Playing { .. } = self.state {
            self.current_beat = self.time / self.level.beat_time();
        }

        let pos = self.cursor_pos.as_f32();
        let game_pos = geng_utils::layout::fit_aabb(
            self.pixel_texture.size().as_f32(),
            Aabb2::ZERO.extend_positive(self.framebuffer_size.as_f32()),
            vec2(0.5, 0.5),
        );
        let pos = pos - game_pos.bottom_left();
        let pos = self
            .model
            .camera
            .screen_to_world(game_pos.size(), pos)
            .as_r32();
        self.cursor_world_pos = if self.snap_to_grid {
            self.snap_pos_grid(pos)
        } else {
            pos
        };

        self.render_lights();
    }

    fn handle_event(&mut self, event: geng::Event) {
        self.handle_event(event);
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.draw(framebuffer);
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
