mod leaderboard;
mod options;
mod profile;
mod sync;

pub mod widget {
    pub use super::{leaderboard::*, options::*, profile::*, sync::*};
    pub use ctl_ui::widget::*;
}

use self::widget::*;
use crate::prelude::*;
pub use ctl_ui::{widget::*, *};
