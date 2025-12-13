mod music;
mod sfx;

pub use self::{music::*, sfx::*};

use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use ctl_assets::{Assets, Options};
use ctl_client::Nertboard;
use ctl_core::{
    prelude::{Id, Time},
    types::FloatTime,
};
use ctl_local::{LevelCache, LocalMusic};
use geng::prelude::{time::Duration, *};

pub const OPTIONS_STORAGE: &str = "options";

// TODO: different id for demo
#[cfg(feature = "steam")]
const STEAM_APP_ID: u32 = 4209820;

#[derive(Clone)]
pub struct Context {
    #[cfg(feature = "steam")]
    pub steam: Option<steamworks::Client>,
    pub geng: Geng,
    pub assets: Rc<Assets>,
    pub music: Rc<MusicManager>,
    pub sfx: Rc<SfxManager>,
    pub local: Rc<LevelCache>,
    options: Rc<RefCell<Options>>,
}

impl Context {
    pub async fn new(
        geng: &Geng,
        assets: &Rc<Assets>,
        client: Option<&Arc<Nertboard>>,
        fs: Rc<ctl_local::fs::Controller>,
    ) -> Result<Self> {
        let options = Rc::new(RefCell::new(
            preferences::load(crate::OPTIONS_STORAGE).unwrap_or_default(),
        ));
        Ok(Self {
            #[cfg(feature = "steam")]
            steam: connect_steam(),
            geng: geng.clone(),
            assets: assets.clone(),
            music: Rc::new(MusicManager::new(geng.clone())),
            sfx: Rc::new(SfxManager::new(geng.clone(), options.clone())),
            local: Rc::new(LevelCache::load(client, fs, geng).await?),
            options,
        })
    }

    /// Expected to be called every frame to maintain relevant global state.
    pub fn update(&self, _delta_time: FloatTime) {
        #[cfg(feature = "steam")]
        if let Some(steam) = &self.steam {
            steam.run_callbacks();
        }
    }

    pub fn get_options(&self) -> Options {
        self.options.borrow().clone()
    }

    pub fn set_options(&self, options: Options) {
        let mut old = self.options.borrow_mut();
        if *old != options {
            preferences::save(crate::OPTIONS_STORAGE, &options);
            *old = options;
        }
    }
}

#[cfg(feature = "steam")]
fn connect_steam() -> Option<steamworks::Client> {
    match steamworks::Client::init_app(STEAM_APP_ID) {
        Ok(steam) => Some(steam),
        Err(err) => {
            log::error!("failed to connect to steam: {}", err);
            log::debug!("{:?}", err);
            None
        }
    }
}
