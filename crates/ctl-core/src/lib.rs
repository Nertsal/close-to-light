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

    pub type Color = Rgba<f32>;
}

use crate::prelude::*;

pub type Score = i32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScoreEntry {
    pub player: String,
    pub score: Score,
    pub extra_info: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: i32,
    /// Secret key used to authenticate.
    pub key: String,
    pub name: String,
}
