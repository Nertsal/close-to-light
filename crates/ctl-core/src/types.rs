use geng::prelude::*;

pub type Id = u32;
pub type Time = R32;
pub type Coord = R32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GroupInfo {
    /// Id `0` for local groups.
    #[serde(default)]
    pub id: Id,
    pub music: MusicInfo,
    pub levels: Vec<LevelInfo>,
}

#[derive(geng::asset::Load, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[load(serde = "toml")]
pub struct MusicInfo {
    /// Id `0` for local music.
    #[serde(default)]
    pub id: Id,
    pub public: bool,
    pub original: bool,
    pub name: String,
    pub bpm: R32,
    pub authors: Vec<ArtistInfo>,
}

impl Default for MusicInfo {
    fn default() -> Self {
        Self {
            id: 0,
            public: false,
            original: false,
            name: "<name>".into(),
            bpm: r32(60.0),
            authors: Vec::new(),
        }
    }
}

impl MusicInfo {
    /// Returns the duration (in seconds) of a single beat.
    pub fn beat_time(&self) -> Time {
        r32(60.0) / self.bpm
    }

    /// Return the list of authors in a readable string format.
    pub fn authors(&self) -> String {
        let authors = self.authors.iter().map(|author| author.name.as_str());
        itertools::Itertools::intersperse(authors, ", ").collect()
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LevelInfo {
    /// Id `0` for local levels.
    #[serde(default)]
    pub id: Id,
    pub name: String,
    pub authors: Vec<UserInfo>,
    #[serde(default)] // TODO: remove
    pub hash: String,
}

impl LevelInfo {
    /// Return the list of authors in a readable string format.
    pub fn authors(&self) -> String {
        let authors = self.authors.iter().map(|author| author.name.as_str());
        itertools::Itertools::intersperse(authors, ", ").collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserInfo {
    pub id: Id,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtistInfo {
    pub id: Id,
    pub name: String,
    pub user: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewMusic {
    pub name: String,
    pub original: bool,
    pub bpm: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicUpdate {
    pub name: Option<String>,
    pub public: Option<bool>,
    pub original: Option<bool>,
    pub bpm: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewLevel {
    pub name: String,
    pub group: Id,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelUpdate {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewArtist {
    pub name: String,
    pub user: Option<Id>,
}
