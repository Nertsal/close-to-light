mod assets;
mod game;
mod model;
mod render;

use geng::prelude::*;

#[derive(clap::Parser)]
struct Opts {
    #[clap(flatten)]
    geng: geng::CliArgs,
}

fn main() {
    logger::init();
    geng::setup_panic_handler();

    let opts: Opts = clap::Parser::parse();

    let geng = Geng::new_with(geng::ContextOptions {
        title: "Geng Game".to_string(),
        ..geng::ContextOptions::from_args(&opts.geng)
    });

    let future = {
        let geng = geng.clone();
        async move {
            let manager = geng.asset_manager();
            let assets = assets::Assets::load(manager).await.unwrap();
            game::Game::new(&geng, &Rc::new(assets))
        }
    };
    geng.run_loading(future)
}
