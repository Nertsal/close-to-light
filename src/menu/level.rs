mod ui;

pub use self::ui::*;

use super::*;

use crate::render::menu::MenuRender;

use geng::MouseButton;

pub struct LevelMenu {
    geng: Geng,
    assets: Rc<Assets>,
    transition: Option<geng::state::Transition>,
    render: MenuRender,

    last_delta_time: Time,

    ui: MenuUI,
    cursor_pos: vec2<f64>,
    state: MenuState,
}

#[derive(Debug)]
pub struct MenuState {
    pub theme: Theme,
    pub groups: Vec<GroupEntry>,
    /// Currently showing group.
    pub show_group: Option<ShowGroup>,
    /// Switch to the group after current one finishes its animation.
    pub switch_group: Option<usize>,
    play_level: Option<LevelId>,
}

#[derive(Debug, Clone)]
pub struct ShowGroup {
    pub group: usize,
    pub time: Bounded<Time>,
}

pub struct GroupEntry {
    pub meta: GroupMeta,
    pub logo: Option<ugli::Texture>,
    /// Path to the group directory.
    pub path: std::path::PathBuf,
}

impl Debug for GroupEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GroupEntry")
            .field("meta", &self.meta)
            .field("logo", &self.logo.as_ref().map(|_| "<logo>"))
            .field("path", &self.path)
            .finish()
    }
}

impl MenuState {
    fn show_group(&mut self, group: usize) {
        self.switch_group = Some(group);
    }

    fn play_level(&mut self, level: LevelId) {
        self.play_level = Some(level);
    }
}

impl LevelMenu {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, groups: Vec<GroupEntry>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            transition: None,
            render: MenuRender::new(geng, assets),

            last_delta_time: Time::ONE,

            ui: MenuUI::new(),
            cursor_pos: vec2::ZERO,

            state: MenuState {
                theme: Theme::default(),
                groups,
                show_group: None,
                switch_group: None,
                play_level: None,
            },
        }
    }

    fn play_level(&mut self, level: LevelId) {
        let future = {
            let geng = self.geng.clone();
            let assets = self.assets.clone();
            let player_name: String = preferences::load(PLAYER_NAME_STORAGE).unwrap_or_default();

            async move {
                let manager = geng.asset_manager();
                let assets_path = run_dir().join("assets");
                let level_path = level.get_path();

                let config: Config =
                    geng::asset::Load::load(manager, &assets_path.join("config.ron"), &())
                        .await
                        .expect("failed to load config");

                let (level_music, level) = load_level(manager, &level_path)
                    .await
                    .expect("failed to load level");
                let level_music = Rc::new(level_music);

                let secrets: Option<crate::Secrets> =
                    geng::asset::Load::load(manager, &run_dir().join("secrets.toml"), &())
                        .await
                        .ok();
                let secrets = secrets.or_else(|| {
                    Some(crate::Secrets {
                        leaderboard: crate::LeaderboardSecrets {
                            id: option_env!("LEADERBOARD_ID")?.to_string(),
                            key: option_env!("LEADERBOARD_KEY")?.to_string(),
                        },
                    })
                });

                crate::game::Game::new(
                    &geng,
                    &assets,
                    config,
                    level,
                    level_music,
                    secrets.map(|s| s.leaderboard),
                    player_name,
                    Time::ZERO,
                )
            }
        };
        self.transition = Some(geng::state::Transition::Push(Box::new(
            geng::LoadingScreen::new(
                &self.geng,
                geng::EmptyLoadingScreen::new(&self.geng),
                future,
            ),
        )));
    }
}

impl geng::State for LevelMenu {
    fn transition(&mut self) -> Option<geng::state::Transition> {
        self.transition.take()
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(self.state.theme.dark), None, None);

        self.ui.layout(
            &mut self.state,
            Aabb2::ZERO.extend_positive(framebuffer.size().as_f32()),
            self.cursor_pos.as_f32(),
            geng_utils::key::is_key_pressed(self.geng.window(), [MouseButton::Left]),
            self.last_delta_time.as_f32(),
            &self.geng,
        );
        self.render.draw_ui(&self.ui, &self.state, framebuffer);

        if let Some(level) = self.state.play_level.take() {
            self.play_level(level);
        }
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::KeyPress {
                key: geng::Key::Escape,
            } => if self.state.switch_group.take().is_some() {},
            geng::Event::CursorMove { position } => {
                self.cursor_pos = position;
            }
            _ => (),
        }
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as f32);

        if let Some(current_group) = &mut self.state.show_group {
            if let Some(switch_group) = self.state.switch_group {
                if current_group.group != switch_group {
                    current_group.time.change(-delta_time);
                    if current_group.time.is_min() {
                        // Switch
                        current_group.group = switch_group;
                    }
                } else {
                    current_group.time.change(delta_time);
                }
            } else {
                current_group.time.change(-delta_time);
                if current_group.time.is_min() {
                    // Remove
                    self.state.show_group = None;
                }
            }
        } else if let Some(group) = self.state.switch_group.take() {
            self.state.show_group = Some(ShowGroup {
                group,
                time: Bounded::new_zero(r32(0.25)),
            });
        }

        self.last_delta_time = delta_time;
    }
}
