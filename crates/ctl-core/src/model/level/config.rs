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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct HealthConfig {
    /// Max health value.
    pub max: FloatTime,
    /// How fast health decreases per second in darkness.
    pub dark_decrease_rate: FloatTime,
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
}

impl LevelModifiers {
    /// Iterate over active modifiers.
    pub fn iter(&self) -> impl Iterator<Item = Modifier> {
        [
            self.nofail.then_some(Modifier::NoFail),
            self.sudden.then_some(Modifier::Sudden),
            self.hidden.then_some(Modifier::Hidden),
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, enum_iterator::Sequence)]
pub enum Modifier {
    NoFail,
    Sudden,
    Hidden,
}

impl Modifier {
    pub fn multiplier(&self) -> R32 {
        match self {
            Modifier::NoFail => r32(0.8),
            Modifier::Sudden => r32(1.15),
            Modifier::Hidden => r32(1.1),
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Modifier::NoFail => "failure is impossible",
            Modifier::Sudden => "the lights are less predictable",
            Modifier::Hidden => "the lights are hidden in the dark",
        }
    }
}

impl Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Modifier::NoFail => write!(f, "Nofail"),
            Modifier::Sudden => write!(f, "Sudden"),
            Modifier::Hidden => write!(f, "Hidden"),
        }
    }
}

impl LevelModifiers {
    pub fn get_mut(&mut self, modifier: Modifier) -> &mut bool {
        match modifier {
            Modifier::NoFail => &mut self.nofail,
            Modifier::Sudden => &mut self.sudden,
            Modifier::Hidden => &mut self.hidden,
        }
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
        }
    }
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self { radius: r32(0.5) }
    }
}

impl HealthConfig {
    // pub fn preset_easy() -> Self {
    //     Self {
    //         max: r32(1.0),
    //         dark_decrease_rate: r32(0.3),
    //         danger_decrease_rate: r32(0.5),
    //         restore_rate: r32(0.5),
    //     }
    // }

    pub fn preset_normal() -> Self {
        Self {
            max: r32(1.0),
            dark_decrease_rate: r32(0.6),
            danger_decrease_rate: r32(0.75),
            restore_rate: r32(0.35),
        }
    }

    // pub fn preset_hard() -> Self {
    //     Self {
    //         max: r32(1.0),
    //         dark_decrease_rate: r32(1.0),
    //         danger_decrease_rate: r32(2.0),
    //         restore_rate: r32(0.25),
    //     }
    // }
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self::preset_normal()
    }
}
