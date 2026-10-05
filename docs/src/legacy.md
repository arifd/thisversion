# Adopting existing persisted data

An existing application may already have persisted data without an explicit
schema version. Over time, its reader may have accumulated aliases, defaults,
and custom decoding rules to keep that data working.

Preserve that behavior as a legacy schema. Here we will call the struct `V0`.
`V0` becomes the compatibility boundary for existing data: its job is to
interpret what was already in storage so it can enter the versioned history.

## Preserve what the old reader knows

V0 does not need to reconstruct a history that was never recorded. Treat it as
a compatibility reader for the legacy formats you can actually identify.

If two old records have the same representation but different meanings, no
schema definition can determine which interpretation was intended. The
application must provide some other information to distinguish them.

## Select the legacy reader explicitly

A family wrapper cannot discover a version tag that was never written. Before
decoding, identify legacy data using information you already have, such as its
source, file location, database metadata, or an unambiguous format marker.

Once V0 has been selected, it can enter the ordinary migration path:

```rust
# extern crate serde_json;
# extern crate serde;
# extern crate thisversion;
# use thisversion::{VersionBoundary, VersionFamily};
#
# struct Settings {
#     name: String,
# }
#
# #[derive(VersionBoundary)]
# #[thisversion(model = Settings)]
# #[derive(serde::Deserialize)]
# struct V0 {
#     name: String,
# }
#
# #[derive(VersionFamily)]
# enum Versions {
#     V0(V0),
# }
use thisversion::traits::IntoModel;

fn read_legacy(input: &str) -> Result<Settings, serde_json::Error> {
    let legacy: V0 = serde_json::from_str(input)?;
    Ok(Versions::V0(legacy).into_model())
}
```

If legacy and versioned data share an entry point, determine the format first
and then decode it with the appropriate reader.

## Establish the new write format

Reading legacy data and writing new data are separate decisions. Once the
application has a current model, retain the version identifier, one way to do this is to serialize it through the family so newly written data carries an explicit schema identity:

```rust
# extern crate serde_json;
# extern crate serde;
# extern crate thisversion;
# use thisversion::{VersionBoundary, VersionFamily};
#
# struct Settings {
#     name: String,
# }
#
# #[derive(VersionBoundary)]
# #[thisversion(model = Settings)]
# #[derive(serde::Serialize)]
# struct V1 {
#     name: String,
# }
#
# #[derive(serde::Serialize, VersionFamily)]
# enum Versions {
#     V1(V1),
# }
use thisversion::traits::IntoSchema;

fn write_current(settings: Settings) -> Result<String, serde_json::Error> {
    let schema: Versions = settings.into_schema();
    serde_json::to_string(&schema)
}
```

From this point on, new records identify their schema explicitly and can follow
the normal versioned migration path.

If you serialize a bare schema instead, store its version separately—for
example, in a database column. The important part is that a future reader can
determine which schema should decode the record.

## Decide when to rewrite legacy data

Loading a legacy record does not require immediately rewriting it. You can
migrate records gradually as they are loaded and saved, or rewrite existing
storage in a separate migration operation.

Whichever strategy you choose, keep the legacy reader for as long as supported
inputs may still arrive in that format. That may include old records that have
not yet been touched, as well as imports, restored backups, or data produced by
older deployments.

Before new writers begin emitting the versioned format, make sure any readers
that must continue working can understand it.
