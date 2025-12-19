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
            geng: geng.clone(),
            assets: assets.clone(),
            music: Rc::new(MusicManager::new(geng.clone())),
            sfx: Rc::new(SfxManager::new(geng.clone(), options.clone())),
            local: Rc::new(LevelCache::load(client, fs, geng).await?),
            options,
            status: Rc::new(RefCell::new(Vec::new())),
        })
    }

    /// Set new active game status.
    pub fn set_status(&self, status: impl Into<String>) {
        #[cfg(not(feature = "steam"))]
        let _ = status; // Noop

        #[cfg(feature = "steam")]
        if let Some(steam) = &self.steam {
            let status = status.into();
            log::debug!("Setting steam status to {:?}", status);
            if steam
                .friends()
                .set_rich_presence("steam_display", Some("#StatusFull"))
                && steam.friends().set_rich_presence("text", Some(&status))
            {
                self.status.borrow_mut().push(status);
            } else {
                log::error!("Failed to set steam status");
            }
        }
    }

    /// Return to the previous game status.
    pub fn pop_status(&self) {
        #[cfg(feature = "steam")]
        if let Some(steam) = &self.steam {
            let mut status = self.status.borrow_mut();
            status.pop();
            let status = status.last().map(|x| x.as_str());
            log::debug!("Setting steam status to {:?}", status);
            steam
                .friends()
                .set_rich_presence("steam_display", status.map(|_| "#StatusFull"));
            steam.friends().set_rich_presence("text", status);
        }
    }

    #[cfg(feature = "steam")]
    pub fn connect_steam(&mut self, steam: steamworks::Client) {
        self.steam = Some(steam);
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
