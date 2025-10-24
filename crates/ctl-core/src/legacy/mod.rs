/// Version as of June 21, 2024 (Mapping Update).
pub mod v1;
/// Version as of October 24th, 2025 (Visual Update).
pub mod v2;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum VersionedLevelSet {
    // V2(v2::LevelSet),
    V2(crate::LevelSet),
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum VersionedLevelSetInfo {
    // V2(v2::LevelSetInfo),
    V2(crate::LevelSetInfo),
}

impl VersionedLevelSet {
    pub fn latest(value: crate::LevelSet) -> Self {
        Self::V2(value)
    }
}

impl VersionedLevelSetInfo {
    pub fn latest(value: crate::LevelSetInfo) -> Self {
        Self::V2(value)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum MigrationError {
    #[error("Level data and metadata have mismatching versions")]
    VersionMismatch,
}

pub fn migrate(
    value: VersionedLevelSet,
    info: VersionedLevelSetInfo,
) -> Result<(crate::LevelSet, crate::LevelSetInfo), MigrationError> {
    match (value, info) {
        // (VersionedLevelSet::V2(value), VersionedLevelSetInfo::V2(info)) => {
        //     Ok(v2::migrate(value, info))
        // }
        (VersionedLevelSet::V2(value), VersionedLevelSetInfo::V2(info)) => Ok((value, info)),
        // _ => Err(MigrationError::VersionMismatch),
    }
}
