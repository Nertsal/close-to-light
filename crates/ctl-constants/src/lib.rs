// TODO: different id for demo
#[cfg(feature = "steam")]
mod steam {
    pub const STEAM_APP_ID: u32 = 4209820;

    pub const STEAM_SERVER_IDENTITY: &str = "close-to-server";
}

#[cfg(feature = "steam")]
pub use self::steam::*;
