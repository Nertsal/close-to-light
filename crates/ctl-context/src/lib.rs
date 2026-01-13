#[cfg(not(target_arch = "wasm32"))]
mod discord;
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
use ctl_local::{Achievements, LevelCache, LocalMusic};
use geng::prelude::{time::Duration, *};

pub const OPTIONS_STORAGE: &str = "options";

#[derive(Clone)]
pub struct Context {
    #[cfg(feature = "steam")]
    pub steam: Option<steamworks::Client>,
    #[cfg(not(target_arch = "wasm32"))]
    pub discord: Option<discord::Client>,

    pub geng: Geng,
    pub assets: Rc<Assets>,
    pub music: Rc<MusicManager>,
    pub sfx: Rc<SfxManager>,
    pub local: Rc<LevelCache>,
    pub achievements: Achievements,
    options: Rc<RefCell<Options>>,
    /// Stack of status, that partially mimicks state transitions.
    status: Rc<RefCell<Vec<String>>>,
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
            steam: None,
            #[cfg(not(target_arch = "wasm32"))]
            discord: None,

            geng: geng.clone(),
            assets: assets.clone(),
            music: Rc::new(MusicManager::new(geng.clone())),
            sfx: Rc::new(SfxManager::new(geng.clone(), options.clone())),
            local: Rc::new(LevelCache::load(client, fs, geng).await?),
            achievements: Achievements::new(),
            options,
            status: Rc::new(RefCell::new(Vec::new())),
        })
    }

    #[cfg(feature = "steam")]
    pub fn connect_steam(&mut self, steam: steamworks::Client) {
        self.achievements.connect_steam(steam.clone());
        self.steam = Some(steam);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn connect_discord(&mut self, discord: discord::Client) {
        self.discord = Some(discord);
    }

    fn set_status_impl(&self, status: Option<&str>) {
        log::debug!("Setting game rich presence to {:?}", status);

        #[cfg(feature = "steam")]
        if let Some(steam) = &self.steam {
            steam
                .friends()
                .set_rich_presence("steam_display", status.map(|_| "#StatusFull"));
            steam.friends().set_rich_presence("text", status);
        }

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(discord) = &self.discord {
            discord.set_status(status);
        }
    }

    /// Set new active game status.
    pub fn set_status(&self, status: impl Into<String>) {
        let status = status.into();
        self.set_status_impl(Some(&status));
        self.status.borrow_mut().push(status);
    }

    /// Return to the previous game status.
    pub fn pop_status(&self) {
        let mut status = self.status.borrow_mut();
        status.pop();
        let status = status.last().map(|x| x.as_str());
        self.set_status_impl(status);
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
pub fn connect_steam() -> Option<steamworks::Client> {
    match steamworks::Client::init_app(ctl_constants::STEAM_APP_ID_CLIENT) {
        Ok(steam) => Some(steam),
        Err(err) => {
            log::error!("failed to connect to steam: {}", err);
            log::debug!("{:?}", err);
            None
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn connect_discord() -> Option<discord::Client> {
    Some(discord::Client::new())
}
