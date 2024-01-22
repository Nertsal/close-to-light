mod assets;
mod command;
mod editor;
mod game;
mod leaderboard;
mod media;
mod menu;
mod model;
mod prelude;
mod render;
mod task;
mod ui;
mod util;

use command::Command;
use leaderboard::Leaderboard;
use prelude::Options;

use std::path::PathBuf;

use geng::prelude::*;

const FIXED_FPS: f64 = 60.0;

const PLAYER_NAME_STORAGE: &str = "close-to-light-name";
const PLAYER_STORAGE: &str = "player";
const OPTIONS_STORAGE: &str = "options";
const HIGHSCORES_STORAGE: &str = "highscores";

#[derive(clap::Parser)]
struct Opts {
    #[command(subcommand)]
    command: Option<Command>,
    /// Skip intro screen.
    #[clap(long)]
    skip_intro: bool,
    /// Play a specific level.
    #[clap(long)]
    level: Option<std::path::PathBuf>,
    /// Move through the level without player input.
    #[clap(long)]
    clean_auto: bool,
    /// Open a level in the editor.
    #[clap(long)]
    edit: bool,
    #[clap(flatten)]
    geng: geng::CliArgs,
}

#[derive(geng::asset::Load, Deserialize, Clone)]
#[load(serde = "toml")]
pub struct Secrets {
    leaderboard: LeaderboardSecrets,
}

#[derive(Deserialize, Clone)]
pub struct LeaderboardSecrets {
    url: String,
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

        let options: Options = preferences::load(OPTIONS_STORAGE).unwrap_or_default();

        let secrets: Option<Secrets> =
            geng::asset::Load::load(manager, &run_dir().join("secrets.toml"), &())
                .await
                .ok();
        let secrets = secrets.or_else(|| {
            Some(Secrets {
                leaderboard: LeaderboardSecrets {
                    url: option_env!("LEADERBOARD_URL")?.to_string(),
                    key: option_env!("LEADERBOARD_KEY")?.to_string(),
                },
            })
        });

        if let Some(command) = opts.command {
            command
                .execute(geng, assets, secrets)
                .await
                .expect("failed to execute the command");
        } else if let Some(level_path) = opts.level {
            let mut config = model::LevelConfig::default();
            let (group_meta, level_meta, music, level) = menu::load_level(manager, &level_path)
                .await
                .expect("failed to load level");

            if opts.edit {
                // Editor
                let editor_config: editor::EditorConfig =
                    geng::asset::Load::load(manager, &assets_path.join("editor.ron"), &())
                        .await
                        .expect("failed to load editor config");

                let level = game::PlayLevel {
                    level_path,
                    group_meta,
                    level_meta,
                    config,
                    level,
                    music,
                    start_time: prelude::Time::ZERO,
                };

                let state =
                    editor::EditorState::new(geng.clone(), assets, editor_config, options, level);
                geng.run_state(state).await;
            } else {
                // Game
                config.modifiers.clean_auto = opts.clean_auto;
                let level = game::PlayLevel {
                    level_path,
                    group_meta,
                    level_meta,
                    config,
                    level,
                    music,
                    start_time: prelude::Time::ZERO,
                };
                let state = game::Game::new(
                    &geng,
                    &assets,
                    options,
                    level,
                    Leaderboard::new(None),
                    "".to_string(),
                );
                geng.run_state(state).await;
            }
        } else {
            // Main menu
            if opts.skip_intro {
                let assets_path = run_dir().join("assets");
                let groups_path = assets_path.join("groups");

                let groups = menu::load_groups(manager, &groups_path)
                    .await
                    .expect("failed to load groups");

                let state = menu::LevelMenu::new(&geng, &assets, groups, secrets, options);
                geng.run_state(state).await;
            } else {
                let state = menu::SplashScreen::new(&geng, &assets, secrets, options);
                geng.run_state(state).await;
            }
        }
    });
}
