mod assets;
mod editor;
mod game;
mod leaderboard;
mod menu;
mod model;
mod prelude;
mod render;
mod ui;
mod util;

use geng::prelude::*;

const FIXED_FPS: f64 = 60.0;

#[derive(clap::Parser)]
struct Opts {
    /// Play a specific level.
    #[clap(long)]
    level: Option<std::path::PathBuf>,
    /// Open a level in the editor.
    #[clap(long)]
    edit: bool,
    #[clap(flatten)]
    geng: geng::CliArgs,
}

#[derive(geng::asset::Load, Deserialize)]
#[load(serde = "toml")]
struct Secrets {
    leaderboard: LeaderboardSecrets,
}

#[derive(Deserialize, Clone)]
pub struct LeaderboardSecrets {
    id: String,
    key: String,
}

fn main() {
    logger::init();
    geng::setup_panic_handler();

    let opts: Opts = batbox::cli::parse();

    let mut options = geng::ContextOptions::default();
    options.window.title = "Geng Game".to_string();
    options.window.antialias = false;
    options.fixed_delta_time = 1.0 / FIXED_FPS;
    options.with_cli(&opts.geng);

    Geng::run_with(&options, move |geng| async move {
        let manager = geng.asset_manager();
        let assets_path = run_dir().join("assets");

        let assets = assets::Assets::load(manager).await.unwrap();
        let assets = Rc::new(assets);

        if let Some(level_path) = opts.level {
            let config: model::Config =
                geng::asset::Load::load(manager, &assets_path.join("config.ron"), &())
                    .await
                    .expect("failed to load config");

            let (level_music, level) = menu::load_level(manager, &level_path)
                .await
                .expect("failed to load level");
            let level_music = Rc::new(level_music);

            if opts.edit {
                // Editor
                let editor_config: editor::EditorConfig =
                    geng::asset::Load::load(manager, &assets_path.join("editor.ron"), &())
                        .await
                        .expect("failed to load editor config");
                let state = editor::EditorState::new(
                    geng.clone(),
                    assets,
                    editor_config,
                    config,
                    level,
                    level_music,
                    level_path,
                );
                geng.run_state(state).await;
            } else {
                // Game
                let state = game::Game::new(
                    &geng,
                    &assets,
                    config,
                    level,
                    level_music,
                    None,
                    "".to_string(),
                    prelude::Time::ZERO,
                );
                geng.run_state(state).await;
            }
        } else {
            // Main menu
            let state = menu::MainMenu::new(&geng, &assets);
            geng.run_state(state).await;
        }
    });
}
