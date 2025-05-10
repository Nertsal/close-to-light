mod music;

pub use self::music::*;

use crate::{
    local::{LevelCache, LocalMusic},
    prelude::{Assets, Id, Options, Time},
};

use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use ctl_client::Nertboard;
use geng::prelude::{time::Duration, *};

#[derive(Clone)]
pub struct Context {
    pub geng: Geng,
    pub assets: Rc<Assets>,
    pub music: Rc<MusicManager>,
    pub local: Rc<LevelCache>,
    options: Rc<RefCell<Options>>,
}

impl Context {
    pub async fn new(
        geng: &Geng,
        assets: &Rc<Assets>,
        client: Option<&Arc<Nertboard>>,
    ) -> Result<Self> {
        Ok(Self {
            geng: geng.clone(),
            assets: assets.clone(),
            music: Rc::new(MusicManager::new(geng.clone())),
            local: Rc::new(LevelCache::load(client, geng).await?),
            options: Rc::new(RefCell::new(
                preferences::load(crate::OPTIONS_STORAGE).unwrap_or_default(),
            )),
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
