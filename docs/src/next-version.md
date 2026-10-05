# Adding the next schema version

Imagine an application that saves a worker's settings between runs. Its first
release stores the worker's name and always uses a 30-second timeout. Later,
users need to choose their own timeout. The application must still understand
the settings it saved before that option existed.

This is a reason to add a schema version. V1 continues to describe records
that contain only a name. V2 stores the chosen timeout as well. A migration
gives V1 records the behavior they had when they were written.

## Extend the history

Keep V1's serialized representation and meaning intact so its stored records
remain readable. Append V2 to the enum deriving
[`VersionFamily`][VersionFamily]. The order of the variants tells the library
which migration runs next.

The application model now needs a timeout too. Move
[`VersionBoundary`][VersionBoundary] and its `model` attribute from V1 to V2, while
V1 keeps its Serde derives. Finally, implement the conversion from V1 to V2.
Together, the types look like this:

```rust
# extern crate serde;
# extern crate thisversion;
use serde::{Deserialize, Serialize};
use thisversion::{VersionBoundary, VersionFamily};

#[derive(Deserialize, Serialize, VersionFamily)]
#[serde(tag = "version", content = "data")]
enum SettingsVersions {
    V1(SettingsV1),
    V2(SettingsV2),
}

#[derive(Deserialize, Serialize)]
struct SettingsV1 {
    name: String,
}

#[derive(Deserialize, Serialize, VersionBoundary)]
#[thisversion(model = Settings)]
struct SettingsV2 {
    name: String,
    timeout_secs: u32,
}

struct Settings {
    name: String,
    timeout_secs: u32,
}

impl From<SettingsV1> for SettingsV2 {
    fn from(previous: SettingsV1) -> Self {
        Self {
            name: previous.name,
            timeout_secs: 30,
        }
    }
}
```

The `30` belongs in the migration because it describes historical behavior.
For example, a stored V1 record containing `{"name":"worker"}` becomes a
`Settings` model with `timeout_secs = 30`. A V2 record has its own
`timeout_secs` value; if that field is missing, the V2 record is invalid.

When you call `into_model()` on a V1 family value, the library first runs the
V1-to-V2 migration, then constructs `Settings`. Future versions extend this
chain with another adjacent migration.

Saving a model through `SettingsVersions` with `into_schema()` now produces V2.
If older applications must read newly written data, update those readers to
recognize V2 before enabling V2 writes.

A new version can also be necessary when field types stay the same but their
meaning changes. Each migration should preserve the interpretation of its
source version. If the new representation cannot accept every old value, see
[Handling failures and model differences](./failures.md).

## Test the persisted contract

Keep serialized fixtures from supported releases. Verify that decoding and
migration produce the intended current values. Constructing an old Rust struct
tests the migration, but not whether bytes written by an old release still
deserialize correctly.

For this change, verify that:

- **V1 settings** load with `timeout_secs = 30`.
- **V2 settings with a custom timeout** are preserved.
- **Newly written settings** use the `V2` tag.

Keep existing serialized tags stable as well. Renaming a Rust variant can change
its serialized identifier unless the codec mapping preserves it explicitly.

[VersionFamily]: https://docs.rs/thisversion/latest/thisversion/derive.VersionFamily.html
[VersionBoundary]: https://docs.rs/thisversion/latest/thisversion/derive.VersionBoundary.html
