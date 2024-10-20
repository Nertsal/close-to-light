use geng::prelude::*;

pub type Id = u32; // TODO: migrate to u64
pub type Time = i64;
pub type FloatTime = R32;
pub type Coord = R32;
pub type Name = Arc<str>;

/// How many time units there are in a single second.
/// 1000 means that each time unit is a millisecond.
pub const TIME_IN_FLOAT_TIME: Time = 1000;

pub fn convert_time(time: FloatTime) -> Time {
    (time.as_f32() / TIME_IN_FLOAT_TIME as f32).round() as Time
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BeatTime {
    /// 1 unit is 1/16 of a beat (typically a 1/64th note).
    units: Time,
}

impl BeatTime {
    /// From whole beats.
    pub fn from_beats(beats: Time) -> Self {
        Self { units: beats * 64 }
    }

    /// From quarter beats.
    pub fn from_4ths(quarters: Time) -> Self {
        Self {
            units: quarters * 16,
        }
    }

    /// From 1/16th beats.
    pub fn from_16ths(units: Time) -> Self {
        Self { units }
    }

    pub fn as_millis(&self, beat_time: Time) -> Time {
        self.units * beat_time / 16
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LevelSet<L = Rc<LevelFull>> {
    pub id: Id,
    pub music: Id,
    pub owner: UserInfo,
    pub levels: Vec<L>,
}

impl<T: Serialize> LevelSet<T> {
    pub fn calculate_hash(&self) -> String {
        let bytes = cbor4ii::serde::to_vec(Vec::new(), self).expect("group should be serializable");
        crate::util::calculate_hash(&bytes)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LevelFull {
    pub meta: LevelInfo,
    pub data: crate::Level,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GroupInfo {
    /// Id `0` for local groups.
    #[serde(default)]
    pub id: Id,
    pub music: MusicInfo,
    pub owner: UserInfo,
    pub levels: Vec<LevelInfo>,
    pub hash: String,
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
    pub bpm: FloatTime,
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

        itertools::Itertools::intersperse(authors.into_iter(), ", ").collect::<String>()
    }
}

impl MusicInfo {
    /// Returns the duration (in seconds) of a single beat.
    pub fn beat_time(&self) -> FloatTime {
        r32(60.0) / self.bpm
    }

    /// Return the list of authors in a readable string format.
    pub fn authors(&self) -> String {
        let authors = self.authors.iter().map(|author| author.name.as_ref());
        itertools::Itertools::intersperse(authors, ", ").collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct LevelInfo {
    /// Id `0` for local levels.
    pub id: Id,
    pub name: Name,
    pub authors: Vec<UserInfo>,
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
pub struct LevelUpdate {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewArtist {
    pub name: String,
    pub romanized_name: String,
    pub user: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupsQuery {
    pub recommended: bool,
}
