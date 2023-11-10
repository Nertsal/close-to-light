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

    ui: MenuUI,
    cursor_pos: vec2<f64>,
    state: MenuState,
}

#[derive(Debug)]
pub struct MenuState {
    pub theme: Theme,
    pub groups: Vec<GroupEntry>,
    pub show_group: Option<usize>,
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

impl LevelMenu {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, groups: Vec<GroupEntry>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            transition: None,
            render: MenuRender::new(geng, assets),

            ui: MenuUI::new(),
            cursor_pos: vec2::ZERO,

            state: MenuState {
                theme: Theme::default(),
                groups,
                show_group: None,
            },
        }
    }

    fn show_group(&mut self, group: usize) {
        self.state.show_group = Some(group);
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
            &self.state,
            Aabb2::ZERO.extend_positive(framebuffer.size().as_f32()),
            self.cursor_pos.as_f32(),
            geng_utils::key::is_key_pressed(self.geng.window(), [MouseButton::Left]),
            &self.geng,
        );
        self.render.draw_ui(&self.ui, &self.state, framebuffer);
    }

    fn handle_event(&mut self, event: geng::Event) {
        if let geng::Event::CursorMove { position } = event {
            self.cursor_pos = position;
        }
    }

    fn update(&mut self, delta_time: f64) {
        let _delta_time = Time::new(delta_time as _);

        if let Some(i) = self.ui.groups.iter().position(|group| group.state.clicked) {
            if let Some(group) = self.state.groups.get(i) {
                // TODO: select level and options and stuff
                let level = LevelId {
                    group: group.path.clone(),
                    level: LevelVariation::Normal,
                };
                self.play_level(level);
            }
        } else if let Some(i) = self.ui.groups.iter().position(|group| group.state.hovered) {
            self.show_group(i);
        }
    }
}
