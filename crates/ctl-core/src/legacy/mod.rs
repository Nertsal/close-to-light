/// Version as of June 21, 2024 (Mapping Update).
pub mod v1;
/// Version as of October 24th, 2025 (Visual Update).
pub mod v2;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum VersionedLevelSet {
    V3(crate::LevelSet),
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum VersionedLevelSetInfo {
    V3(crate::LevelSetInfo),
}

impl VersionedLevelSet {
    pub fn latest(value: crate::LevelSet) -> Self {
        Self::V3(value)
    }
}

impl VersionedLevelSetInfo {
    pub fn latest(value: crate::LevelSetInfo) -> Self {
        Self::V3(value)
    }
}

pub fn migrate(
    value: VersionedLevelSet,
    info: VersionedLevelSetInfo,
) -> (crate::LevelSet, crate::LevelSetInfo) {
    match (value, info) {
        (VersionedLevelSet::V3(value), VersionedLevelSetInfo::V3(info)) => (value, info),
    }
}
