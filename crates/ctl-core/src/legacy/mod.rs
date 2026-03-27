/// Version as of June 21st, 2024 (Mapping Update).
pub mod v1;
/// Version as of October 24th, 2025 (Visual Update).
pub mod v2;
/// Version as of February 20th, 2026 (Demo Release).
pub mod v3;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum VersionedLevelSet {
    V2(v2::LevelSet),
    // V3(v3::LevelSet),
    V3(crate::LevelSet),
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum VersionedLevelSetInfo {
    V2(v2::LevelSetInfo),
    // V3(v3::LevelSetInfo),
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

#[derive(thiserror::Error, Debug, Clone)]
pub enum LevelMigrationError {
    #[error("Found mismatching versions of data and metadata, migration is not possible")]
    MismatchingVersions,
}

pub fn migrate(
    value: VersionedLevelSet,
    info: VersionedLevelSetInfo,
) -> Result<(crate::LevelSet, crate::LevelSetInfo), LevelMigrationError> {
    match (value, info) {
        (VersionedLevelSet::V2(value), VersionedLevelSetInfo::V2(info)) => {
            Ok(v2::migrate(value, info))
        }
        (VersionedLevelSet::V3(value), VersionedLevelSetInfo::V3(info)) => Ok((value, info)),
        _ => Err(LevelMigrationError::MismatchingVersions),
    }
}
