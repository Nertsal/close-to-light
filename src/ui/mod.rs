mod leaderboard;
mod options;
mod pause;
mod profile;
mod score;
#[cfg(feature = "online")]
mod sync;

pub mod widget {
    #[cfg(feature = "online")]
    pub use super::sync::*;
    pub use super::{leaderboard::*, options::*, pause::*, profile::*, score::*};
    pub use ctl_ui::widget::*;
}

use self::widget::*;
use crate::prelude::*;
pub use ctl_ui::{widget::*, *};
