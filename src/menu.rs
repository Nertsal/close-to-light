mod level;
mod main;
mod splash;

pub use self::{level::*, main::*, splash::*};

use crate::{
    prelude::*,
    render::{
        dither::DitherRender,
        util::{TextRenderOptions, UtilRender},
    },
    OPTIONS_STORAGE, PLAYER_NAME_STORAGE,
};
