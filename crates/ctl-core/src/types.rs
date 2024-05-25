use geng::prelude::*;

pub type Id = u32;
pub type Time = R32;
pub type Coord = R32;
pub type Name = Arc<str>;

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
    pub name: Name,
    pub romanized: Name,
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
            romanized: "<romanized>".into(),
            bpm: r32(60.0),
            authors: Vec::new(),
        }
    }
}

impl GroupInfo {
    /// Return the list of map authors in a readable string format.
    pub fn mappers(&self) -> String {
        let mut authors: Vec<&str> = self
            .levels
            .iter()
            .flat_map(|level| level.authors.iter().map(|user| user.name.as_ref()))
            .collect();
        authors.sort();
        authors.dedup();

        itertools::Itertools::intersperse(authors.into_iter(), ",").collect::<String>()
    }
}

impl MusicInfo {
    /// Returns the duration (in seconds) of a single beat.
    pub fn beat_time(&self) -> Time {
        r32(60.0) / self.bpm
    }

    /// Return the list of authors in a readable string format.
    pub fn authors(&self) -> String {
        let authors = self.authors.iter().map(|author| author.name.as_ref());
        itertools::Itertools::intersperse(authors, ", ").collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LevelInfo {
    /// Id `0` for local levels.
    #[serde(default)]
    pub id: Id,
    pub name: Name,
    pub authors: Vec<UserInfo>,
    #[serde(default)] // TODO: remove
    pub hash: String,
}

impl Default for LevelInfo {
    fn default() -> Self {
        Self {
            id: 0,
            name: "<level>".into(),
            authors: Vec::new(),
            hash: "".into(),
        }
    }
}

impl LevelInfo {
    /// Return the list of authors in a readable string format.
    pub fn authors(&self) -> String {
        let authors = self.authors.iter().map(|author| author.name.as_ref());
        itertools::Itertools::intersperse(authors, ", ").collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserInfo {
    pub id: Id,
    pub name: Name,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserLogin {
    pub id: Id,
    pub name: Name,
    /// The token that can be used to login later.
    pub token: Name,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtistInfo {
    pub id: Id,
    pub name: Name,
    pub romanized: Name,
    pub user: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewMusic {
    pub name: String,
    pub romanized_name: String,
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
    /// If set, updates an existing level instead of creating a new one.
    pub level_id: Option<Id>,
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
