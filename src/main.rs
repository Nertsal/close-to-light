mod assets;
#[cfg(not(target_arch = "wasm32"))]
mod command;
mod context;
mod editor;
mod game;
mod leaderboard;
mod local;
#[cfg(not(target_arch = "wasm32"))]
mod media;
mod menu;
mod model;
mod prelude;
mod render;
mod task;
mod ui;
mod util;

// use leaderboard::Leaderboard;
use prelude::Options;

use geng::prelude::*;

const FIXED_FPS: f64 = 60.0; // TODO: upgrade to 120 i think

const OPTIONS_STORAGE: &str = "options";
const HIGHSCORES_STORAGE: &str = "highscores";
const PLAYER_LOGIN_STORAGE: &str = "user";

const DISCORD_URL: &str = "https://discord.com/oauth2/authorize?client_id=1242091884709417061&response_type=code&redirect_uri=http%3A%2F%2Flocalhost%3A3000%2Fauth%2Fdiscord&scope=identify";

#[derive(clap::Parser)]
struct Opts {
    #[cfg(not(target_arch = "wasm32"))]
    #[command(subcommand)]
    command: Option<command::Command>,
    /// Skip intro screen.
    #[clap(long)]
    skip_intro: bool,
    // TODO: reimplement
    // /// Play a specific level.
    // #[clap(long)]
    // level: Option<std::path::PathBuf>,
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
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    let mut builder = tokio::runtime::Builder::new_current_thread();

    #[cfg(not(target_arch = "wasm32"))]
    let mut builder = tokio::runtime::Builder::new_multi_thread();

    builder.enable_all().build().unwrap().block_on(async_main());
}

async fn async_main() {
    let mut builder = logger::builder();
    builder.filter_level(log::LevelFilter::Debug);
    logger::init_with(builder).expect("failed to initialize logger");
    geng::setup_panic_handler();

    let opts: Opts = batbox::cli::parse();

    let mut options = geng::ContextOptions::default();
    options.window.title = "Geng Game".to_string();
    options.window.antialias = false;
    options.fixed_delta_time = 1.0 / FIXED_FPS;
    options.with_cli(&opts.geng);

    Geng::run_with(&options, move |geng| async move {
        if let Err(err) = geng_main(opts, geng).await {
            log::error!("{:?}", err);
        }
    });
}

async fn geng_main(opts: Opts, geng: Geng) -> anyhow::Result<()> {
    let manager = geng.asset_manager();
    // let assets_path = run_dir().join("assets");

    let assets = assets::Assets::load(manager).await?;
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
            },
        })
    });
    let client = secrets
        .as_ref()
        .map(|secrets| ctl_client::Nertboard::new(&secrets.leaderboard.url))
        .transpose()?
        .map(Arc::new);

    let context = context::Context::new(&geng, &assets, client.as_ref())
        .await
        .expect("failed to initialize context");

    #[cfg(not(target_arch = "wasm32"))]
    if let Some(command) = opts.command {
        command
            .execute(geng, assets, secrets)
            .await
            .context("failed to execute the command")?;
        return Ok(());
    }

    // if let Some(level_path) = opts.level {
    //     let mut config = model::LevelConfig::default();
    //     let (music, level) = context
    //         .local
    //         .load_level(&level_path)
    //         .await
    //         .context("failed to load the level")?;

    //     if opts.edit {
    //         // Editor
    //         let editor_config: editor::EditorConfig =
    //             geng::asset::Load::load(manager, &assets_path.join("editor.ron"), &())
    //                 .await
    //                 .context("failed to load editor config")?;

    //         let level = game::PlayLevel {
    //             music,
    //             level,
    //             config,
    //             start_time: prelude::Time::ZERO,
    //         };

    //         let state = editor::EditorState::new(context, editor_config, options, level);
    //         geng.run_state(state).await;
    //         return Ok(());
    //     }

    //     // Game
    //     config.modifiers.clean_auto = opts.clean_auto;
    //     let level = game::PlayLevel {
    //         music,
    //         level,
    //         config,
    //         start_time: prelude::Time::ZERO,
    //     };
    //     let state = game::Game::new(context, options, level, Leaderboard::new(&geng, None));
    //     geng.run_state(state).await;
    // } else {

    // Main menu
    if opts.skip_intro {
        let state = menu::LevelMenu::new(context, client.as_ref(), options);
        geng.run_state(state).await;
    } else {
        let state = menu::SplashScreen::new(context, client.as_ref(), options);
        geng.run_state(state).await;
    }

    // }

    Ok(())
}
