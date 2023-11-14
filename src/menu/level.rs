mod ui;

pub use self::ui::*;

use super::*;

use crate::render::menu::MenuRender;

use geng_egui::*;

struct Icons {
    pub title: Icon,
    pub level_frame: Icon,
    pub prev: Icon,
    pub next: Icon,
}

impl Icons {
    pub fn load(ugli: &Ugli, assets: &Assets, context: &egui::Context) -> anyhow::Result<Self> {
        Ok(Self {
            title: Icon::from_ugli(ugli, &assets.sprites.title, context)?,
            level_frame: Icon::from_ugli(ugli, &assets.sprites.level_frame, context)?,
            prev: Icon::from_ugli(ugli, &assets.sprites.button_prev, context)?,
            next: Icon::from_ugli(ugli, &assets.sprites.button_next, context)?,
        })
    }
}

pub struct LevelMenu {
    geng: Geng,
    assets: Rc<Assets>,
    transition: Option<geng::state::Transition>,
    render: MenuRender,

    last_delta_time: Time,

    ui: EguiGeng,
    cursor_pos: vec2<f64>,
    state: MenuState,
}

pub struct MenuState {
    icons: Icons,
    pub theme: Theme,
    pub groups: Vec<GroupEntry>,
    /// Currently showing group.
    pub show_group: Option<ShowGroup>,
    /// Switch to the group after current one finishes its animation.
    pub switch_group: Option<usize>,
    play_level: Option<(LevelId, LevelConfig)>,
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

    fn play_level(&mut self, level: LevelId, config: LevelConfig) {
        self.play_level = Some((level, config));
    }
}

impl LevelMenu {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, groups: Vec<GroupEntry>) -> Self {
        let ui = EguiGeng::new(geng);
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            transition: None,
            render: MenuRender::new(geng, assets),

            last_delta_time: Time::ONE,

            state: MenuState {
                icons: Icons::load(geng.ugli(), assets, ui.get_context())
                    .expect("when loading icons"),
                theme: Theme::default(),
                groups,
                show_group: None,
                switch_group: None,
                play_level: None,
            },

            ui,
            cursor_pos: vec2::ZERO,
        }
    }

    fn play_level(&mut self, level: LevelId, config: LevelConfig) {
        let future = {
            let geng = self.geng.clone();
            let assets = self.assets.clone();
            let player_name: String = preferences::load(PLAYER_NAME_STORAGE).unwrap_or_default();

            async move {
                let manager = geng.asset_manager();
                // let assets_path = run_dir().join("assets");
                let level_path = level.get_path();

                let (_, level_music, level) = load_level(manager, &level_path)
                    .await
                    .expect("failed to load level");

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

        self.ui.draw(framebuffer);

        if let Some((level, config)) = self.state.play_level.take() {
            self.play_level(level, config);
        }
    }

    fn handle_event(&mut self, event: geng::Event) {
        self.ui.handle_event(event.clone());

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

        self.ui.begin_frame();
        self.ui(delta_time);
        self.ui.end_frame();

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
