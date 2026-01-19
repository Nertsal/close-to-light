// ----- Game Version -----

pub const GAME_VERSION: GameVersion = GameVersion {
    major: 0,
    minor: 1,
    patch: 1,
};

#[derive(Debug, Clone, Copy)]
pub struct GameVersion {
    pub major: usize,
    pub minor: usize,
    pub patch: usize,
}

impl std::fmt::Display for GameVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v")?;
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;

        #[cfg(feature = "demo")]
        write!(f, "-demo")?;

        Ok(())
    }
}

// ----- Steam -----

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

// ----- General -----

pub const DISCORD_APP_ID: u64 = 1242091884709417061;
