use super::*;

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct LevelConfig {
    pub player: PlayerConfig,
    pub health: HealthConfig,
    pub modifiers: LevelModifiers,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerConfig {
    pub radius: Coord,
    pub buffer_time: Time,
    pub coyote_time: Time,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct HealthConfig {
    /// Max health value.
    pub max: FloatTime,
    /// How fast health decreases per second in darkness.
    pub dark_decrease_rate: FloatTime,
    /// The initial health penalty for touching a red light.
    pub danger_penalty: FloatTime,
    /// Cooldown between danger penalties.
    pub danger_cooldown: FloatTime,
    /// How fast health decreases per second in danger.
    pub danger_decrease_rate: FloatTime,
    /// How much health restores per second while in light.
    pub restore_rate: FloatTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct LevelModifiers {
    /// Play through the level without player input.
    pub clean_auto: bool,
    /// You cannot fail the level.
    pub nofail: bool,
    /// No telegraphs.
    pub sudden: bool,
    /// Don't render lights.
    pub hidden: bool,
    /// Whether touchscreen was used during gameplay.
    pub touch: bool,
    /// Time speed up or slow down.
    pub time_scale: FloatTime,
    /// Normal/Flashlight/Spotlight.
    pub light: Option<LightMode>,
    /// Difficulty settings of gameplay: health drain, red penalty, coyote time.
    pub difficulty: DifficultyMode,
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DifficultyMode {
    // Reduced difficulty.
    Candle,
    /// Normal difficulty settings.
    #[default]
    Normal,
    /// Increased difficulty.
    Laser,
    /// Max difficulty - touching red is instant death.
    Solar,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LightMode {
    /// Stuff is only visible within a small range near the cursor.
    Flashlight,
    /// Stuff is only visible within a range of lights.
    Spotlight,
}

impl LevelConfig {
    pub fn validate(&mut self) {
        self.health = HealthConfig::preset(self.modifiers.difficulty);
        self.player = PlayerConfig::preset(self.modifiers.difficulty);
    }
}

impl LevelModifiers {
    pub fn get_mut(&mut self, modifier: Modifier) -> Option<&mut bool> {
        match modifier {
            Modifier::NoFail => Some(&mut self.nofail),
            Modifier::Sudden => Some(&mut self.sudden),
            Modifier::Hidden => Some(&mut self.hidden),
            Modifier::Touch => Some(&mut self.touch),
            Modifier::TimeScale(_) => None,
            Modifier::LightMode(_) => None,
            Modifier::Difficulty(_) => None,
        }
    }

    pub fn reset(&mut self, modifier: Modifier) {
        match modifier {
            Modifier::NoFail => self.nofail = false,
            Modifier::Sudden => self.sudden = false,
            Modifier::Hidden => self.hidden = false,
            Modifier::Touch => self.touch = false,
            Modifier::TimeScale(_) => self.time_scale = FloatTime::ONE,
            Modifier::LightMode(_) => self.light = None,
            Modifier::Difficulty(_) => self.difficulty = DifficultyMode::default(),
        }
    }

    /// Iterate over active modifiers.
    pub fn iter(&self) -> impl Iterator<Item = Modifier> {
        [
            self.touch.then_some(Modifier::Touch),
            self.sudden.then_some(Modifier::Sudden),
            self.hidden.then_some(Modifier::Hidden),
            (self.time_scale != FloatTime::ONE).then_some(Modifier::TimeScale(self.time_scale)),
            self.light.map(Modifier::LightMode),
            (self.difficulty != DifficultyMode::Normal)
                .then_some(Modifier::Difficulty(self.difficulty)),
            self.nofail.then_some(Modifier::NoFail),
        ]
        .into_iter()
        .flatten()
    }

    pub fn multiplier(&self) -> R32 {
        r32(self
            .iter()
            .map(|modifier| modifier.multiplier().as_f32())
            .product())
    }
}

#[allow(clippy::derivable_impls)]
impl Default for LevelModifiers {
    fn default() -> Self {
        Self {
            clean_auto: false,
            nofail: false,
            sudden: false,
            hidden: false,
            touch: false,
            time_scale: FloatTime::ONE,
            light: None,
            difficulty: DifficultyMode::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Modifier {
    NoFail,
    Sudden,
    Hidden,
    Touch,
    TimeScale(FloatTime),
    LightMode(LightMode),
    Difficulty(DifficultyMode),
}

impl Modifier {
    /// Whether two modifiers are compatible with each other.
    /// Same variants are considered inconstructible.
    pub fn is_compatible(self, other: Self) -> bool {
        fn compat_assym(a: Modifier, b: Modifier) -> bool {
            if let Modifier::NoFail = a
                && matches!(b, Modifier::Difficulty(_))
            {
                // nofail is not compatible with difficulty changes
                return false;
            }

            true
        }

        compat_assym(self, other) && compat_assym(other, self)
    }

    pub fn multiplier(&self) -> R32 {
        match self {
            Modifier::NoFail => r32(0.8),
            Modifier::Sudden => r32(1.15),
            Modifier::Hidden => r32(1.1),
            Modifier::Touch => r32(1.0),
            &Modifier::TimeScale(scale) => {
                if scale < FloatTime::ONE {
                    (scale - r32(0.4)).clamp(r32(0.1), r32(1.0))
                } else {
                    r32(1.0) + (scale - r32(1.0)) * r32(0.4)
                }
            }
            Modifier::LightMode(LightMode::Flashlight) => r32(1.05),
            Modifier::LightMode(LightMode::Spotlight) => r32(1.05),
            Modifier::Difficulty(DifficultyMode::Candle) => r32(0.9),
            Modifier::Difficulty(DifficultyMode::Normal) => r32(1.0),
            Modifier::Difficulty(DifficultyMode::Laser) => r32(1.1),
            Modifier::Difficulty(DifficultyMode::Solar) => r32(1.15),
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Modifier::NoFail => "failure is impossible",
            Modifier::Sudden => "the lights are less predictable",
            Modifier::Hidden => "the lights are hidden in the dark",
            Modifier::Touch => "played with touchscreen",
            &Modifier::TimeScale(scale) => {
                if scale < FloatTime::ONE {
                    "slow motion"
                } else {
                    "fast motion"
                }
            }
            Modifier::LightMode(LightMode::Flashlight) => {
                "who turned the lights off??\nvision is limited"
            }
            Modifier::LightMode(LightMode::Spotlight) => {
                "the lights are spot on!\nvision is limited"
            }
            Modifier::Difficulty(DifficultyMode::Candle) => "game difficulty is reduced",
            Modifier::Difficulty(DifficultyMode::Normal) => "the intended game experience",
            Modifier::Difficulty(DifficultyMode::Laser) => {
                "laser-like precision is required\nrhythm is tighter"
            }
            Modifier::Difficulty(DifficultyMode::Solar) => "difficulty of the sun\nred means death",
        }
    }
}

impl Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Modifier::NoFail => write!(f, "Nofail"),
            Modifier::Sudden => write!(f, "Sudden"),
            Modifier::Hidden => write!(f, "Hidden"),
            Modifier::Touch => write!(f, "Touch"),
            &Modifier::TimeScale(scale) => {
                if scale < FloatTime::ONE {
                    write!(f, "Half Time")
                } else {
                    write!(f, "Double Time")
                }
            }
            Modifier::LightMode(LightMode::Flashlight) => write!(f, "Flashlight"),
            Modifier::LightMode(LightMode::Spotlight) => write!(f, "Spotlight"),
            Modifier::Difficulty(DifficultyMode::Candle) => write!(f, "Candle"),
            Modifier::Difficulty(DifficultyMode::Normal) => write!(f, "Normal"),
            Modifier::Difficulty(DifficultyMode::Laser) => write!(f, "Laser"),
            Modifier::Difficulty(DifficultyMode::Solar) => write!(f, "Solar"),
        }
    }
}

impl PlayerConfig {
    pub fn preset(mode: DifficultyMode) -> Self {
        let (radius, buffer_time, coyote_time) = match mode {
            DifficultyMode::Candle => (0.5, 80, 80),
            DifficultyMode::Normal => (0.5, 80, 80),
            DifficultyMode::Laser => (0.45, 50, 50),
            DifficultyMode::Solar => (0.45, 50, 50),
        };
        Self {
            radius: r32(radius),
            buffer_time: TIME_IN_FLOAT_TIME * buffer_time / 1000,
            coyote_time: TIME_IN_FLOAT_TIME * coyote_time / 1000,
        }
    }
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self::preset(DifficultyMode::default())
    }
}

impl HealthConfig {
    pub fn preset(mode: DifficultyMode) -> Self {
        match mode {
            DifficultyMode::Candle => Self::preset_candle(),
            DifficultyMode::Normal => Self::preset_normal(),
            DifficultyMode::Laser => Self::preset_laser(),
            DifficultyMode::Solar => Self::preset_solar(),
        }
    }

    /// Easy mode.
    pub fn preset_candle() -> Self {
        Self {
            max: r32(1.0),
            dark_decrease_rate: r32(0.5),
            danger_penalty: r32(0.2),
            danger_cooldown: r32(0.6),
            danger_decrease_rate: r32(0.7),
            restore_rate: r32(0.5),
        }
    }

    /// Normal mode.
    pub fn preset_normal() -> Self {
        Self {
            max: r32(1.0),
            dark_decrease_rate: r32(0.7),
            danger_penalty: r32(0.2),
            danger_cooldown: r32(0.6),
            danger_decrease_rate: r32(1.3),
            restore_rate: r32(0.4),
        }
    }

    /// Hard mode.
    pub fn preset_laser() -> Self {
        Self {
            max: r32(1.0),
            dark_decrease_rate: r32(0.85),
            danger_penalty: r32(0.3),
            danger_cooldown: r32(0.6),
            danger_decrease_rate: r32(2.0),
            restore_rate: r32(0.25),
        }
    }

    /// Impossible mode.
    pub fn preset_solar() -> Self {
        Self {
            max: r32(1.0),
            dark_decrease_rate: r32(1.0),
            danger_penalty: r32(1.0),
            danger_cooldown: r32(0.6),
            danger_decrease_rate: r32(2.0),
            restore_rate: r32(0.2),
        }
    }
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self::preset(DifficultyMode::default())
    }
}
