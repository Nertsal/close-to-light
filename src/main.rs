#![cfg_attr(not(any(debug_assertions)), windows_subsystem = "windows")]

#[cfg(not(target_arch = "wasm32"))]
mod command;
mod editor;
mod game;
#[cfg(not(target_arch = "wasm32"))]
mod media;
mod menu;
mod prelude;
mod render;
mod ui;
mod util;

use ctl_client::Nertboard;
use ctl_context::Context;
use geng::prelude::*;

const FIXED_FPS: f64 = 60.0; // TODO: upgrade to 120 i think

const DISCORD_SERVER_URL: &str = "https://discord.gg/Aq9bTvSbFN";

#[derive(clap::Parser)]
struct Opts {
    #[clap(long)]
    log: Option<String>,
    #[cfg(not(target_arch = "wasm32"))]
    #[command(subcommand)]
    command: Option<command::Command>,
    /// Skip intro screen.
    #[clap(long)]
    skip_intro: bool,
    /// Move through the level without player input.
    #[clap(long)]
    clean_auto: bool,
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
    let opts: Opts = batbox::cli::parse();

    let mut builder = logger::builder();
    builder
        .filter_level(
            if let Some(level) = opts.log.as_deref().or(option_env!("LOG")) {
                match level {
                    "trace" => log::LevelFilter::Trace,
                    "debug" => log::LevelFilter::Debug,
                    "info" => log::LevelFilter::Info,
                    "warn" => log::LevelFilter::Warn,
                    "error" => log::LevelFilter::Error,
                    "off" => log::LevelFilter::Off,
                    _ => panic!("invalid log level string"),
                }
            } else if cfg!(debug_assertions) {
                log::LevelFilter::Debug
            } else {
                log::LevelFilter::Info
            },
        )
        .filter_module("calloop", log::LevelFilter::Debug)
        .filter_module("discord_presence", log::LevelFilter::Off);
    logger::init_with(builder).expect("failed to init logger");
    geng::setup_panic_handler();

    log::info!("Running Close to Light {}", ctl_constants::GAME_VERSION);

    let mut options = geng::ContextOptions::default();
    options.window.title = "Close to Light".to_string();
    options.window.antialias = false;
    options.fixed_delta_time = 1.0 / FIXED_FPS;
    options.with_cli(&opts.geng);

    Geng::run_with(&options, move |geng| async move {
        let main = geng_main(geng, opts);

        #[cfg(not(target_arch = "wasm32"))]
        let main = async_compat::Compat::new(main);

        if let Err(err) = main.await {
            log::error!("{err:?}");
        }
    });
}

async fn geng_main(geng: Geng, opts: Opts) -> anyhow::Result<()> {
    let loading_assets: Rc<ctl_assets::LoadingAssets> =
        geng::asset::Load::load(geng.asset_manager(), &run_dir().join("assets"), &())
            .await
            .context("when loading assets")?;

    let load_everything = load_everything(geng.clone());

    #[allow(unused_mut)]
    let mut insta_load = opts.skip_intro;
    #[cfg(not(target_arch = "wasm32"))]
    {
        insta_load = insta_load || opts.command.is_some();
    }

    let loading_screen =
        menu::LoadingScreen::new(&geng, loading_assets, load_everything, insta_load).run();

    let (context, secrets, client) = loading_screen
        .await
        .ok_or_else(|| anyhow::Error::msg("loading screen failed"))??;

    #[cfg(not(target_arch = "wasm32"))]
    if let Some(command) = opts.command {
        command
            .execute(context, secrets)
            .await
            .context("failed to execute the command")?;
        return Ok(());
    }

    let _ = secrets;

    // Main menu
    if opts.skip_intro {
        let leaderboard = ctl_local::Leaderboard::new(
            &geng,
            client.as_ref(),
            &context.local.fs,
            &context.achievements,
            context.get_options().account.auto_login,
        );
        let state = menu::LevelMenu::new(context, leaderboard, None);
        geng.run_state(state).await;
    } else {
        let state = menu::SplashScreen::new(context, client.as_ref());
        geng.run_state(state).await;
    }

    Ok(())
}

async fn load_everything(
    geng: Geng,
) -> anyhow::Result<(Context, Option<Secrets>, Option<Arc<Nertboard>>)> {
    let manager = geng.asset_manager();

    let assets = ctl_assets::Assets::load(manager).await?;
    let assets = Rc::new(assets);

    let secrets: Option<Secrets> =
        match geng::asset::Load::load(manager, &run_dir().join("secrets.toml"), &()).await {
            Ok(secrets) => {
                log::debug!("Successfully loaded secrets.toml");
                Some(secrets)
            }
            Err(err) => {
                log::debug!("Failed to load secrets.toml: {err:?}");
                None
            }
        };
    let secrets = secrets.or_else(|| {
        let url = option_env!("LEADERBOARD_URL");
        if url.is_none() {
            log::debug!("LEADERBOARD_URL environment variable is not set, launching offline");
            return None;
        }
        log::debug!("Loaded LEADERBOARD_URL");
        Some(Secrets {
            leaderboard: LeaderboardSecrets {
                url: url?.to_string(),
            },
        })
    });

    #[allow(unused_mut)] // used with only some features
    let mut client = secrets
        .as_ref()
        .map(|secrets| ctl_client::Nertboard::new(&secrets.leaderboard.url))
        .transpose()?;
    if let Some(client) = &client {
        let _ = client.ping().await; // Ping the server to check if we are online
    }

    #[cfg(feature = "steam")]
    let steam = {
        let steam = ctl_context::connect_steam();
        if let Some(steam) = &steam
            && let Some(client) = &mut client
        {
            client.connect_steam(steam.clone());
        }
        steam
    };

    let client = client.map(Arc::new);

    let fs = Rc::new(
        ctl_local::fs::Controller::new(&geng)
            .await
            .expect("failed to initialize file system"),
    );

    #[allow(unused_mut)] // used with only some features
    let mut context = Context::new(&geng, &assets, client.as_ref(), fs)
        .await
        .expect("failed to initialize context");

    #[cfg(feature = "steam")]
    if let Some(steam) = steam {
        context.connect_steam(steam.clone());
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let discord = ctl_context::connect_discord();
        if let Some(discord) = discord {
            context.connect_discord(discord);
        }
    }

    Ok((context, secrets, client))
}
