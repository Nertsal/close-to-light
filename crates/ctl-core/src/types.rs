use geng::prelude::*;
use geng_utils::bounded::Bounded;

pub type Id = u32; // TODO: migrate to u64
pub type Time = i64;
pub type FloatTime = R32;
pub type Coord = R32;
pub type Name = Arc<str>;
pub type Lifetime = Bounded<FloatTime>;

/// How many time units there are in a single second.
/// 1000 means that each time unit is a millisecond.
pub const TIME_IN_FLOAT_TIME: Time = 1000;
pub const COYOTE_TIME: Time = TIME_IN_FLOAT_TIME / 10; // 0.1s
pub const BUFFER_TIME: Time = TIME_IN_FLOAT_TIME / 10; // 0.1s

pub fn seconds_to_time(time: impl Float) -> Time {
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
    pub const UNITS_PER_BEAT: Time = 120;

    /// A whole beat (typically a 1/4th note).
    pub const WHOLE: Self = Self {
        units: Self::UNITS_PER_BEAT,
    };
    /// A half beat (typically a 1/8th note).
    pub const HALF: Self = Self {
        units: Self::UNITS_PER_BEAT / 2,
    };
    /// A third beat (typically a 1/12th note).
    pub const THIRD: Self = Self {
        units: Self::UNITS_PER_BEAT / 3,
    };
    /// A quarter beat (typically a 1/16th note).
    pub const QUARTER: Self = Self {
        units: Self::UNITS_PER_BEAT / 4,
    };
    /// A fifth beat (typically a 1/20th note).
    pub const FIFTH: Self = Self {
        units: Self::UNITS_PER_BEAT / 5,
    };
    /// An eighth beat (typically a 1/32th note).
    pub const EIGHTH: Self = Self {
        units: Self::UNITS_PER_BEAT / 8,
    };
    pub const ZERO: Self = Self { units: 0 };

    /// From whole beats.
    pub fn from_beats(beats: Time) -> Self {
        Self {
            units: beats * Self::UNITS_PER_BEAT,
        }
    }

    /// Approximate exact beat time from fractional beats.
    pub fn from_beats_float(beats: FloatTime) -> Self {
        Self {
            units: (beats.as_f32() * Self::UNITS_PER_BEAT as f32).round() as Time,
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
pub struct LevelSet<L = Rc<crate::Level>> {
    pub levels: Vec<L>,
}

impl<T: Serialize> LevelSet<T> {
    pub fn calculate_hash(&self) -> String {
        let bytes = cbor4ii::serde::to_vec(Vec::new(), self).expect("group should be serializable");
        crate::util::calculate_hash(&bytes)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelFull<L = Rc<crate::Level>> {
    pub meta: LevelInfo,
    pub data: L,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelSetFull {
    pub meta: LevelSetInfo,
    pub data: LevelSet<crate::Level>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LevelSetInfo {
    /// Id `0` for local groups.
    #[serde(default)]
    pub id: Id,
    pub music: MusicInfo,
    pub owner: UserInfo,
    #[serde(default)]
    pub levels: Vec<LevelInfo>,
    #[serde(default)]
    pub featured: bool,
    pub hash: String,
}

#[derive(geng::asset::Load, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[load(serde = "toml")]
pub struct MusicInfo {
    /// Id `0` for local music.
    #[serde(default)]
    pub id: Id,
    #[serde(default)]
    pub original: bool,
    #[serde(default)]
    pub featured: bool,
    pub name: Name,
    pub romanized: Name,
    #[serde(default)]
    pub authors: Vec<MusicianInfo>,
}

impl Default for MusicInfo {
    fn default() -> Self {
        Self {
            id: 0,
            original: false,
            featured: false,
            name: "<name>".into(),
            romanized: "<romanized>".into(),
            authors: Vec::new(),
        }
    }
}

impl LevelSetInfo {
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
    pub authors: Vec<MapperInfo>,
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

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
pub struct MapperInfo {
    /// User id `0` for non-registered mapper.
    pub id: Id,
    pub name: Name,
    pub romanized: Name,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MusicianInfo {
    /// Id `0` for non-registered musicians.
    pub id: Id,
    pub name: Name,
    pub romanized: Name,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewMusic {
    pub name: String,
    pub romanized_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicUpdate {
    pub name: Option<String>,
    pub original: Option<bool>,
    pub featured: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelUpdate {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewMusician {
    pub name: String,
    pub romanized_name: String,
    pub user: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelSetsQuery {
    pub recommended: bool,
}

pub fn non_zero(id: Id) -> Option<Id> {
    if id == 0 { None } else { Some(id) }
}
