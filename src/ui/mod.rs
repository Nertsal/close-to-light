mod leaderboard;
mod options;
mod profile;
mod score;
mod sync;

pub mod widget {
    pub use super::{leaderboard::*, options::*, profile::*, score::*, sync::*};
    pub use ctl_ui::widget::*;
}

use self::widget::*;
use crate::prelude::*;
pub use ctl_ui::{widget::*, *};
