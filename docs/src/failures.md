# Fallible conversions

Use `TryFrom` when a migration can reject old values. Keep both derives: mark
the family `fallible`, give the latest schema an error type, and load with
`try_into_model()`.

Here, a retry count narrows from `u32` to `u8`. The standard conversion supplies
`TryFromIntError` when the old value exceeds 255:

```rust
# extern crate serde;
# extern crate serde_json;
# extern crate thisversion;
use std::num::TryFromIntError;
use serde::{Deserialize, Serialize};
use thisversion::{VersionBoundary, VersionFamily};
use thisversion::traits::{IntoSchema, TryIntoModel};

#[derive(Deserialize, Serialize, VersionFamily)]
#[serde(tag = "version", content = "data")]
#[thisversion(fallible)]
enum SettingsVersions {
    V1(SettingsV1),
    V2(SettingsV2),
}

#[derive(Deserialize, Serialize)]
struct SettingsV1 {
    retries: u32,
}

#[derive(Deserialize, Serialize, VersionBoundary)]
#[thisversion(model = Settings, error = TryFromIntError)]
struct SettingsV2 {
    retries: u8,
}

struct Settings {
    retries: u8,
}

impl TryFrom<SettingsV1> for SettingsV2 {
    type Error = TryFromIntError;

    fn try_from(old: SettingsV1) -> Result<Self, Self::Error> {
        Ok(Self { retries: old.retries.try_into()? })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = r#"{"version":"V1","data":{"retries":3}}"#;
    let stored: SettingsVersions = serde_json::from_str(input)?;
    let settings = stored.try_into_model()?;
    let stored: SettingsVersions = settings.into_schema();
    assert!(matches!(stored, SettingsVersions::V2(_)));
    Ok(())
}
```

## Nested errors

The latest schema chooses the family's final error type. Errors from migration
steps and nested conversions must convert into it through `From`.

A parent can reuse the child's error. Add these types to the example above:

```rust
# extern crate serde;
# extern crate serde_json;
# extern crate thisversion;
# use serde::Deserialize;
# struct Settings {
#     retries: u8,
# }
#
# #[derive(Deserialize)]
# struct SettingsV1 {
#     retries: u32,
# }
#
# #[derive(VersionBoundary, Deserialize)]
# #[thisversion(model = Settings, error = TryFromIntError)]
# struct SettingsV2 {
#     retries: u8,
# }
#
# impl TryFrom<SettingsV1> for SettingsV2 {
#     type Error = TryFromIntError;
#
#     fn try_from(old: SettingsV1) -> Result<Self, Self::Error> {
#         Ok(Self { retries: old.retries.try_into()? })
#     }
# }
#
# #[derive(Deserialize, VersionFamily)]
# #[serde(tag = "version", content = "data")]
# #[thisversion(fallible)]
# enum SettingsVersions {
#     V1(SettingsV1),
#     V2(SettingsV2),
# }
use std::num::TryFromIntError;
use thisversion::{VersionBoundary, VersionFamily};
use thisversion::traits::{IntoSchema, TryIntoModel};

struct Job {
    settings: Settings,
}

#[derive(VersionBoundary)]
#[thisversion(model = Job, error = TryFromIntError)]
struct JobV1 {
    #[thisversion(nested)]
    settings: SettingsVersions,
}
```

`JobV1::try_into_model()` propagates a failed settings migration. If the parent
has several error sources, choose a parent error enum and implement `From` for
each source. Mark a containing version family `fallible` too. Infallible `From`
migrations can remain in a fallible family; their `Infallible` error must also
convert into the final error type.

## Validation and error handling

Latest inputs skip historical migrations. Here, V2's `u8` already prevents
out-of-range retry counts. Other invariants may need checks during decoding or
[custom model construction](./manual.md). `error = E` enables error propagation;
it does not add validation to ordinary fields. Check invalid historical and
latest inputs, including nested values.

The Serde `DeserializeIntoModel` adapter combines decoding and migration, turning
conversion errors into Serde errors using `Display`. To retain domain errors
for matching, decode first, as above. To return decoding and migration errors
through one application error, add:

```rust
# extern crate serde;
# extern crate serde_json;
# extern crate thisversion;
# use serde::Deserialize;
# struct Settings {
#     retries: u8,
# }
#
# #[derive(Deserialize)]
# struct SettingsV1 {
#     retries: u32,
# }
#
# #[derive(VersionBoundary, Deserialize)]
# #[thisversion(model = Settings, error = TryFromIntError)]
# struct SettingsV2 {
#     retries: u8,
# }
#
# impl TryFrom<SettingsV1> for SettingsV2 {
#     type Error = TryFromIntError;
#
#     fn try_from(old: SettingsV1) -> Result<Self, Self::Error> {
#         Ok(Self { retries: old.retries.try_into()? })
#     }
# }
#
# #[derive(VersionFamily, Deserialize)]
# #[serde(tag = "version", content = "data")]
# #[thisversion(fallible)]
# enum SettingsVersions {
#     V1(SettingsV1),
#     V2(SettingsV2),
# }
use std::num::TryFromIntError;

use thisversion::{VersionBoundary, VersionFamily};
use thisversion::traits::{IntoSchema, TryIntoModel};

#[derive(Debug)]
enum LoadError {
    Decode(serde_json::Error),
    Migration(TryFromIntError),
}

fn load(input: &str) -> Result<Settings, LoadError> {
    let stored: SettingsVersions =
        serde_json::from_str(input).map_err(LoadError::Decode)?;
    stored.try_into_model().map_err(LoadError::Migration)
}
```
