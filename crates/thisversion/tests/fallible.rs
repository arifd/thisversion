use thisversion::traits::TryIntoModel;
use thisversion::{VersionBoundary, VersionFamily};

#[derive(Debug, PartialEq, Eq)]
enum MigrationError {
    EmptyName,
    ZeroTimeout,
}

#[derive(VersionFamily)]
#[thisversion(fallible)]
enum SettingsVersions {
    V1(SettingsV1),
    V2(SettingsV2),
    V3(SettingsV3),
}

struct SettingsV1 {
    name: String,
    timeout_secs: u32,
}

struct SettingsV2 {
    name: String,
    timeout_secs: u32,
    enabled: bool,
}

#[derive(VersionBoundary)]
#[thisversion(model = Settings, error = MigrationError)]
struct SettingsV3 {
    name: String,
    timeout_secs: u32,
    enabled: bool,
    retries: u8,
}

impl TryFrom<SettingsV1> for SettingsV2 {
    type Error = MigrationError;

    fn try_from(old: SettingsV1) -> Result<Self, Self::Error> {
        if old.name.trim().is_empty() {
            return Err(MigrationError::EmptyName);
        }

        Ok(Self {
            name: old.name,
            timeout_secs: old.timeout_secs,
            enabled: true,
        })
    }
}

impl TryFrom<SettingsV2> for SettingsV3 {
    type Error = MigrationError;

    fn try_from(old: SettingsV2) -> Result<Self, Self::Error> {
        if old.timeout_secs == 0 {
            return Err(MigrationError::ZeroTimeout);
        }

        Ok(Self {
            name: old.name,
            timeout_secs: old.timeout_secs,
            enabled: old.enabled,
            retries: 3,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Settings {
    name: String,
    timeout_secs: u32,
    enabled: bool,
    retries: u8,
}

#[test]
fn oldest_version_runs_both_fallible_migrations() {
    let stored = SettingsVersions::V1(SettingsV1 {
        name: "worker".into(),
        timeout_secs: 30,
    });

    assert_eq!(
        stored.try_into_model(),
        Ok(Settings {
            name: "worker".into(),
            timeout_secs: 30,
            enabled: true,
            retries: 3,
        })
    );
}

#[test]
fn first_migration_reports_its_error() {
    let stored = SettingsVersions::V1(SettingsV1 {
        name: "  ".into(),
        timeout_secs: 30,
    });

    assert_eq!(stored.try_into_model(), Err(MigrationError::EmptyName));
}

#[test]
fn second_migration_reports_its_error_from_either_earlier_version() {
    let oldest = SettingsVersions::V1(SettingsV1 {
        name: "worker".into(),
        timeout_secs: 0,
    });

    // Check V1 passes through the second migration after the first succeeds:
    assert_eq!(oldest.try_into_model(), Err(MigrationError::ZeroTimeout));

    let previous = SettingsVersions::V2(SettingsV2 {
        name: "worker".into(),
        timeout_secs: 0,
        enabled: false,
    });

    // Check V2 enters the same failing transition directly:
    assert_eq!(previous.try_into_model(), Err(MigrationError::ZeroTimeout));
}

#[test]
fn latest_version_skips_historical_migrations() {
    let stored = SettingsVersions::V3(SettingsV3 {
        name: String::new(),
        timeout_secs: 0,
        enabled: false,
        retries: 1,
    });

    assert_eq!(
        stored.try_into_model(),
        Ok(Settings {
            name: String::new(),
            timeout_secs: 0,
            enabled: false,
            retries: 1,
        })
    );
}
