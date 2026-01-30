mod leaderboard;
mod options;
mod pause;
mod profile;
mod score;
mod sync;

pub mod widget {
    pub use super::{leaderboard::*, options::*, pause::*, profile::*, score::*, sync::*};
    pub use ctl_ui::widget::*;
}

use self::widget::*;
use crate::prelude::*;
pub use ctl_ui::{widget::*, *};
