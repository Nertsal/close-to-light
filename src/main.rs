mod assets;
mod editor;
mod game;
mod model;
mod render;
mod util;

use geng::prelude::*;

#[derive(clap::Parser)]
struct Opts {
    /// Open a level in the editor.
    #[clap(long)]
    edit: bool,
    #[clap(flatten)]
    geng: geng::CliArgs,
}

fn main() {
    logger::init();
    geng::setup_panic_handler();

    let opts: Opts = clap::Parser::parse();

    let mut options = geng::ContextOptions::default();
    options.window.title = "Geng Game".to_string();
    options.with_cli(&opts.geng);

    Geng::run_with(&options, move |geng| async move {
        let manager = geng.asset_manager();
        let assets_path = run_dir().join("assets");

        let assets = assets::Assets::load(manager).await.unwrap();
        let assets = Rc::new(assets);
        let level: model::Level =
            geng::asset::Load::load(manager, &assets_path.join("level.json"), &())
                .await
                .expect("failed to load level");
        let config: model::Config =
            geng::asset::Load::load(manager, &assets_path.join("config.ron"), &())
                .await
                .expect("failed to load config");

        if opts.edit {
            // Editor
            let editor_config: editor::EditorConfig =
                geng::asset::Load::load(manager, &assets_path.join("editor.ron"), &())
                    .await
                    .expect("failed to load editor config");
            let state = editor::Editor::new(geng.clone(), assets, editor_config, config, level);
            geng.run_state(state).await;
        } else {
            // Game
            let state = game::Game::new(&geng, &assets, config, level, model::Time::ZERO);
            geng.run_state(state).await;
        }
    });
}
