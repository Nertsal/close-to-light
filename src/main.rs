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

#[derive(clap::Parser)]
struct Opts {
    /// Open a level in the editor.
    #[clap(long)]
    edit: Option<std::path::PathBuf>,
    /// Play a specific level.
    #[clap(long)]
    level: Option<std::path::PathBuf>,
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
    options.with_cli(&opts.geng);

    Geng::run_with(&options, move |geng| async move {
        let manager = geng.asset_manager();
        let assets_path = run_dir().join("assets");

        let assets = assets::Assets::load(manager).await.unwrap();
        let assets = Rc::new(assets);
        let config: model::Config =
            geng::asset::Load::load(manager, &assets_path.join("config.ron"), &())
                .await
                .expect("failed to load config");

        if let Some(level_path) = opts.edit {
            // Editor
            // let level_path = assets_path.join("levels").join("level.json");
            let level: model::Level = geng::asset::Load::load(manager, &level_path, &())
                .await
                .expect("failed to load level");

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
                level_path,
            );
            geng.run_state(state).await;
        } else if let Some(level_path) = opts.level {
            // Game
            let level: model::Level = geng::asset::Load::load(manager, &level_path, &())
                .await
                .expect("failed to load level");
            let state = game::Game::new(
                &geng,
                &assets,
                config,
                level,
                None,
                "".to_string(),
                prelude::Time::ZERO,
            );
            geng.run_state(state).await;
        } else {
            // Main menu
            let state = menu::MainMenu::new(&geng, &assets, config);
            geng.run_state(state).await;
        }
    });
}
