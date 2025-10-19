mod music;
mod sfx;

pub use self::{music::*, sfx::*};

use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use ctl_assets::{Assets, Options};
use ctl_client::Nertboard;
use ctl_core::prelude::{Id, Time};
use ctl_local::{LevelCache, LocalMusic};
use geng::prelude::{time::Duration, *};

pub const OPTIONS_STORAGE: &str = "options";

#[derive(Clone)]
pub struct Context {
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
            geng: geng.clone(),
            assets: assets.clone(),
            music: Rc::new(MusicManager::new(geng.clone())),
            sfx: Rc::new(SfxManager::new(geng.clone(), options.clone())),
            local: Rc::new(LevelCache::load(client, fs, geng).await?),
            options,
        })
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
