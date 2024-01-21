pub mod model;
pub mod types;

pub mod prelude {
    pub use crate::{model::*, types::*};

    pub use std::collections::VecDeque;

    pub use geng::prelude::*;
    pub use geng_utils::{
        bounded::Bounded,
        conversions::{RealConversions, Vec2RealConversions},
    };
    pub use serde::{Deserialize, Serialize};
    pub use uuid::Uuid;

    pub type Color = Rgba<f32>;
}

use crate::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScoreEntry {
    pub player: PlayerInfo,
    pub score: i32,
    pub extra_info: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: Id,
    /// Secret key used to authenticate.
    pub key: String,
    pub name: String,
}
