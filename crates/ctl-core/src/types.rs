use geng::prelude::*;

pub type Id = u32; // TODO: migrate to u64
pub type Time = i64;
pub type FloatTime = R32;
pub type Coord = R32;
pub type Name = Arc<str>;

/// How many time units there are in a single second.
/// 1000 means that each time unit is a millisecond.
pub const TIME_IN_FLOAT_TIME: Time = 1000;

pub fn seconds_to_time(time: FloatTime) -> Time {
    (time.as_f32() * TIME_IN_FLOAT_TIME as f32).round() as Time
}

pub fn time_to_seconds(time: Time) -> FloatTime {
    FloatTime::new(time as f32 / TIME_IN_FLOAT_TIME as f32)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
struct BeatTimeSerde(FloatTime);

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(from = "BeatTimeSerde", into = "BeatTimeSerde")]
pub struct BeatTime {
    /// 1 unit is 1/16 of a beat (typically a 1/64th note).
    // TODO: do 1/48 or 1/60 to support thirds
    units: Time,
}

impl From<BeatTime> for BeatTimeSerde {
    fn from(value: BeatTime) -> Self {
        Self(r32(value.units as f32 / BeatTime::UNITS_PER_BEAT as f32))
    }
}

impl From<BeatTimeSerde> for BeatTime {
    fn from(value: BeatTimeSerde) -> Self {
        Self {
            units: (value.0.as_f32() * BeatTime::UNITS_PER_BEAT as f32).round() as Time,
        }
    }
}

impl BeatTime {
    pub const UNITS_PER_BEAT: Time = 16;
    /// A whole beat (typically a 1/4th note)
    pub const WHOLE: Self = Self {
        units: Self::UNITS_PER_BEAT,
    };
    /// A half beat (typically a 1/8th note)
    pub const HALF: Self = Self {
        units: Self::UNITS_PER_BEAT / 2,
    };
    /// A quarter beat (typically a 1/16th note)
    pub const QUARTER: Self = Self {
        units: Self::UNITS_PER_BEAT / 4,
    };
    /// An eighth beat (typically a 1/32th note)
    pub const EIGHTH: Self = Self {
        units: Self::UNITS_PER_BEAT / 8,
    };

    /// From whole beats.
    pub fn from_beats(beats: Time) -> Self {
        Self {
            units: beats * Self::UNITS_PER_BEAT,
        }
    }

    /// From quarter beats.
    pub fn from_4ths(quarters: Time) -> Self {
        Self {
            units: quarters * (Self::UNITS_PER_BEAT / 4),
        }
    }

    /// From 1/16th beats.
    pub fn from_16ths(units: Time) -> Self {
        Self { units }
    }

    /// From atomic units as defined by [`BeatTime::UNITS_PER_BEAT`].
    pub fn from_units(units: Time) -> Self {
        Self { units }
    }

    pub fn units(&self) -> Time {
        self.units
    }

    pub fn as_beats(&self) -> R32 {
        r32(self.units as f32 / Self::UNITS_PER_BEAT as f32)
    }

    pub fn as_time(&self, beat_time: FloatTime) -> Time {
        seconds_to_time(self.as_secs(beat_time))
    }

    pub fn as_secs(&self, beat_time: FloatTime) -> FloatTime {
        self.as_beats() * beat_time
    }
}

impl Add<Self> for BeatTime {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            units: self.units + rhs.units,
        }
    }
}

impl AddAssign<Self> for BeatTime {
    fn add_assign(&mut self, rhs: Self) {
        self.units += rhs.units;
    }
}

impl Sub<Self> for BeatTime {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            units: self.units - rhs.units,
        }
    }
}

impl SubAssign<Self> for BeatTime {
    fn sub_assign(&mut self, rhs: Self) {
        self.units -= rhs.units;
    }
}

impl Mul<Time> for BeatTime {
    type Output = Self;

    fn mul(self, rhs: Time) -> Self::Output {
        Self {
            units: self.units * rhs,
        }
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
