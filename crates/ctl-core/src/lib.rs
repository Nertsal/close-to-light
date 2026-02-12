pub mod auth;
pub mod interpolation;
pub mod legacy;
pub mod model;
pub mod score;
pub mod types;
pub mod util;

pub mod prelude {
    pub use crate::{interpolation::*, model::*, types::*};

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

pub const DISCORD_LOGIN_URL: &str = "https://discord.com/oauth2/authorize?client_id=1242091884709417061&response_type=code&scope=identify";
