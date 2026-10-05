#![no_std]

use thisversion::traits::{IntoModel, IntoSchema, TryIntoModel};
use thisversion::{VersionBoundary, VersionFamily};

pub struct Application {
    pub value: u32,
}

#[derive(VersionBoundary)]
#[thisversion(model = Application)]
pub struct Snapshot {
    pub value: u32,
}

#[derive(VersionFamily)]
pub enum ModelVersions {
    V1(u16),
    Current(Snapshot),
}

impl From<u16> for Snapshot {
    fn from(value: u16) -> Self {
        Self {
            value: u32::from(value),
        }
    }
}

pub fn migrate_model(value: u16) -> Application {
    ModelVersions::V1(value).into_model()
}

pub fn wrap_model(value: Application) -> ModelVersions {
    value.into_schema()
}

#[test]
fn optional_models_convert_without_alloc() {
    let model = Some(ModelVersions::V1(7)).into_model();
    let stored: Option<ModelVersions> = model.into_schema();

    // Check Option composes the family conversions without allocation:
    assert!(matches!(
        stored,
        Some(ModelVersions::Current(Snapshot { value: 7 }))
    ));

    let absent: Option<ModelVersions> = None::<Application>.into_schema();

    // Check missing values survive conversion in both directions:
    assert!(absent.into_model().is_none());
}

pub struct Old(u64);
pub struct FallibleApplication {
    pub value: u32,
}
pub struct MigrationError;

impl From<core::convert::Infallible> for MigrationError {
    fn from(error: core::convert::Infallible) -> Self {
        match error {}
    }
}

impl From<core::num::TryFromIntError> for MigrationError {
    fn from(_: core::num::TryFromIntError) -> Self {
        Self
    }
}

#[derive(VersionBoundary)]
#[thisversion(model = FallibleApplication, error = MigrationError)]
pub struct FallibleSnapshotModel {
    pub value: u32,
}

impl TryFrom<Old> for FallibleSnapshotModel {
    type Error = MigrationError;

    fn try_from(old: Old) -> Result<Self, Self::Error> {
        let value = u32::try_from(old.0).map_err(MigrationError::from)?;
        Ok(Self { value })
    }
}

pub enum FallibleVersions {
    Old(Old),
    Current(FallibleSnapshotModel),
}

impl From<FallibleSnapshotModel> for FallibleVersions {
    fn from(value: FallibleSnapshotModel) -> Self {
        Self::Current(value)
    }
}

impl TryIntoModel for FallibleVersions {
    type Model = FallibleApplication;
    type Error = MigrationError;

    fn try_into_model(self) -> Result<Self::Model, Self::Error> {
        match self {
            Self::Old(old) => FallibleSnapshotModel::try_from(old)?.try_into_model(),
            Self::Current(current) => current.try_into_model(),
        }
    }
}

pub fn migrate_fallible(value: u64) -> Result<FallibleApplication, MigrationError> {
    FallibleVersions::Old(Old(value)).try_into_model()
}

#[test]
fn optional_fallible_migrations_work_without_alloc() {
    let valid = Some(FallibleVersions::Old(Old(7))).try_into_model();

    // Check successful optional migrations produce the application model:
    assert!(matches!(valid, Ok(Some(FallibleApplication { value: 7 }))));

    let invalid = Some(FallibleVersions::Old(Old(u64::MAX))).try_into_model();

    // Check migration errors pass through Option unchanged:
    assert!(matches!(invalid, Err(MigrationError)));

    // Check absent values require no migration:
    assert!(matches!(
        None::<FallibleVersions>.try_into_model(),
        Ok(None)
    ));
}
