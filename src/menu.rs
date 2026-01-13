mod game_preview;
mod level;
mod loading;
mod main;
mod splash;

pub use self::{game_preview::*, level::*, loading::*, main::*, splash::*};

use crate::{
    prelude::*,
    render::{dither::DitherRender, util::UtilRender},
};

use ctl_local::Leaderboard;
use ctl_render_core::TextRenderOptions;
use ctl_util::SecondOrderState;

/// General state structure that holds the configuration data used in the options.
pub struct GameOptions {
    pub context: Context,
    pub leaderboard: Leaderboard,
    /// Interpolated radius of the player collider used to preview cursor outline.
    pub player_size: SecondOrderState<R32>,
    /// Preview of gameplay used to give context to some settings.
    pub preview: GameplayPreview,
}

impl GameOptions {
    pub fn new(context: Context, leaderboard: Leaderboard) -> Self {
        Self {
            context,
            leaderboard,
            player_size: SecondOrderState::new(3.0, 1.0, 0.0, r32(0.1)),
            preview: GameplayPreview::new(),
        }
    }
}
