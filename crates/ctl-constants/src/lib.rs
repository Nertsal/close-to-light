// TODO: different id for demo
#[cfg(feature = "steam")]
mod steam {
    /// App ID of the full game.
    pub const STEAM_APP_ID_FULL: u32 = 4209820;
    /// App ID of the demo version.
    pub const STEAM_APP_ID_DEMO: u32 = 4259120;

    /// App ID of the client version of the game (full or demo).
    #[cfg(feature = "demo")]
    pub const STEAM_APP_ID_CLIENT: u32 = STEAM_APP_ID_DEMO;
    #[cfg(not(feature = "demo"))]
    pub const STEAM_APP_ID_CLIENT: u32 = STEAM_APP_ID_FULL;

    /// Server identity being used in Steam API.
    pub const STEAM_SERVER_IDENTITY: &str = "close-to-server";
}

#[cfg(feature = "steam")]
pub use self::steam::*;
