<div align="center">

<img alt="thisversion logo" src="https://raw.githubusercontent.com/arifd/thisversion/main/docs/assets/thisversion.svg" width="256" height="256">

<h1>Thisversion</h1>

**Keep schema history out of your application model.**

[<img alt="crates.io" src="https://img.shields.io/crates/v/thisversion.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/thisversion)
[<img alt="MSRV 1.88+" src="https://img.shields.io/badge/MSRV-1.88%2B-8da0cb?style=for-the-badge&labelColor=555555" height="20">](https://www.rust-lang.org/)
[<img alt="no_std supported" src="https://img.shields.io/badge/no__std-supported-35b99b?style=for-the-badge&labelColor=555555" height="20">](https://doc.rust-lang.org/stable/embedded-book/intro/no-std.html)
[<img alt="unsafe forbidden" src="https://img.shields.io/badge/unsafe-forbidden-e9ae35?style=for-the-badge&labelColor=555555" height="20">](https://github.com/rust-secure-code/safety-dance/)

<a href="https://arifd.github.io/thisversion/">Book</a> ·
<a href="https://docs.rs/thisversion">Documentation</a> ·
<a href="https://github.com/arifd/thisversion/tree/main/crates/thisversion/examples">Examples</a> ·
<a href="https://github.com/arifd/thisversion/blob/main/CHANGELOG.md">Changelog</a>

</div>

## The Problem

Persisted data often outlives the code that wrote it. As an application evolves,
fields may be renamed, added, removed, or change meaning.

<details open>
<summary>Example: compatibility rules accumulating in a deserializer</summary>

```rust
#[derive(serde::Deserialize)]
struct Settings {
    // Keeping this alias means `label` cannot later be used for a separate
    // field.
    #[serde(alias = "label")]
    name: String,

    // Serde cannot tell whether this field is absent because the data is old
    // or because the current data violates its contract.
    #[serde(default)]
    retries: u8,

    // Older data used seconds; newer data uses milliseconds.
    // Accepting both requires a policy for conflicts and missing values.
    timeout_secs: Option<u32>,
    timeout_ms: Option<u64>,

    // This field's meaning has changed over time.
    status: String,
}
```

</details>

One approach is to make the current deserializer accept every historical shape.
As the schema changes, compatibility rules accumulate, especially for
independently evolving nested types, and the deserialization layer becomes
harder to maintain. Furthermore, without explicit schema versions, the
deserializer cannot know which semantics to apply when meaning changes:
`status: "active"` might have meant that the account was enabled in previous
versions, but now means its subscription is paid and current. Both records
decode to the same Rust type.

`thisversion` gives each schema version its own Rust type, separating the work
of decoding a schema from migrating it into the application model. Explicit
conversions make migrations expressive and independently testable, including
changes in meaning as well as structure. Rust checks the conversions between
the latest schema and application model, so incompatible model changes fail at
compile time. This lets you evolve the model with confidence: when older data
must remain compatible, express the change as a new schema version and
migration.

## Installation

Simply add `thisversion` to your `Cargo.toml`:

```toml
[dependencies]
thisversion = "0.1"
```

For the example below you will also need `serde`, and `serde_json`.

## Features

The optional `serde` feature provides convenience adapters when you want to
deserialize directly into the application model. This is not needed for the
example below.

The optional `alloc` feature allows a `Vec` of versioned data to be transformed
between model and schema. This is not needed for the example below.

## Basic usage

Represent each persisted schema as a Rust type, then list them in a
`VersionFamily` enum from oldest to newest. Derive `VersionBoundary` on the latest
schema to connect the version family to the application model.

Migrations connect each schema version to the next using `From` or `TryFrom`.
The application works with one current model; `thisversion` handles the
versioned representations at the persistence boundary.

<details open>
<summary>Example: migrating settings between versions</summary>

```rust
use std::time::Duration;

use serde::{Deserialize, Serialize};
use thisversion::traits::{IntoModel, IntoSchema};

//===========================================================================//
// APPLICATION                                                               //
//===========================================================================//

// The application model; free of any versioning history.
#[derive(Debug, PartialEq, Eq)]
pub struct Settings {
    pub name: String,
    pub retries: u8,
    pub account_enabled: bool,
    pub subscription_status: SubscriptionStatus,
    pub timeout: Duration,
}

// The application model; free of any versioning history.
#[derive(Debug, PartialEq, Eq)]
pub enum SubscriptionStatus {
    Active,
    PastDue,
    Canceled,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Persisted state acquired from somewhere...
    let persisted = r#"{"V1":{"label":"worker","timeout_secs":15,"status":"active"}}"#;

    // The call to `.into_model()` migrates it into the current application model.
    let settings = serde_json::from_str::<SettingsVersions>(persisted)?.into_model();

    assert_eq!(
        settings,
        Settings {
            name: "worker".into(),
            retries: 0,
            account_enabled: true,
            subscription_status: SubscriptionStatus::Canceled,
            timeout: Duration::from_secs(15),
        }
    );

    // Calling `.into_schema()` gives us back the schema type for persistence.
    let schema: SettingsVersions = settings.into_schema();

    assert!(matches!(schema, SettingsVersions::V3(_)));
    assert_eq!(
        serde_json::to_string(&schema)?,
        r#"{"V3":{"name":"worker","retries":0,"account_enabled":true,"status":{"V1":"canceled"},"timeout_ms":15000}}"#
    );

    Ok(())
}

//===========================================================================//
// VERSIONS                                                                  //
//===========================================================================//

// All this historical complexity can be abstracted away from the day-to-day
// business logic while also gaining maintainability and testability.
//
// A change in the application model will cause a compiler error prompting the
// developer to declare a new version.

use thisversion::{VersionBoundary, VersionFamily};

#[derive(Deserialize, Serialize, VersionFamily)]
enum SettingsVersions {
    V1(SettingsV1),
    V2(SettingsV2),
    V3(SettingsV3),
}

// V1 stored the name as `label`, timeout in seconds, and account state as
// `status`.
#[derive(Deserialize, Serialize)]
struct SettingsV1 {
    #[serde(rename = "label")]
    name: String,
    timeout_secs: u32,
    status: V1AccountStatus,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum V1AccountStatus {
    Active,
    Inactive,
}

// V2 uses `name` and changed the timeout unit, adds retries, and gives
// account state an explicit meaning.
#[derive(Deserialize, Serialize)]
struct SettingsV2 {
    name: String,
    retries: u8,
    timeout_ms: u64,
    account_enabled: bool,
}

// V1 had no retry count; its `active` status meant the account was enabled.
impl From<SettingsV1> for SettingsV2 {
    fn from(previous: SettingsV1) -> Self {
        Self {
            name: previous.name,
            retries: 0,
            timeout_ms: u64::from(previous.timeout_secs) * 1_000,
            account_enabled: matches!(previous.status, V1AccountStatus::Active),
        }
    }
}

// V3 reuses the wire field `status` for subscription state. We also separate
// the wire format of `timeout` from the more useful Rust model.
#[derive(Deserialize, Serialize, VersionBoundary)]
#[thisversion(model = Settings)]
struct SettingsV3 {
    name: String,
    retries: u8,
    account_enabled: bool,
    #[serde(rename = "status")]
    #[thisversion(nested)]
    subscription_status: SubscriptionStatusVersions,
    #[serde(rename = "timeout_ms", with = "duration_millis")]
    timeout: Duration,
}

// Settings had no subscription state, so the migration policy treats it as
// canceled.
impl From<SettingsV2> for SettingsV3 {
    fn from(previous: SettingsV2) -> Self {
        Self {
            name: previous.name,
            retries: previous.retries,
            account_enabled: previous.account_enabled,
            subscription_status: SubscriptionStatusV1::Canceled.into(),
            timeout: Duration::from_millis(previous.timeout_ms),
        }
    }
}

// Subscription state has its own history, so it can evolve independently of
// the containing schema.
#[derive(Deserialize, Serialize, VersionFamily)]
enum SubscriptionStatusVersions {
    V1(SubscriptionStatusV1),
}

// The latest persisted representation maps directly to the application enum.
#[derive(Deserialize, Serialize, VersionBoundary)]
#[thisversion(model = SubscriptionStatus)]
#[serde(rename_all = "snake_case")]
enum SubscriptionStatusV1 {
    Active,
    PastDue,
    Canceled,
}

// The Rust model uses Duration; the wire format uses an integer count of
// milliseconds.
mod duration_millis {
    use std::time::Duration;
    
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let millis = u64::try_from(value.as_millis()).map_err(serde::ser::Error::custom)?;
        serializer.serialize_u64(millis)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        u64::deserialize(deserializer).map(Duration::from_millis)
    }
}
```

</details>

## Persistence

`thisversion` is agnostic about the storage system, wire format, and version
identifier. Persisted data only needs a way to select its schema. In the
previous example, Serde's `V1`, `V2`, or `V3` tag selects the schema; Serde
decodes its fields, and `thisversion` migrates it into the current application
model. Serde is optional: migrations can be used with other serialization
formats or storage systems.

The optional `serde` feature also provides adapters for deserializing directly
into the application model.

## Nesting

Large persisted structures often contain parts with different rates of change.
If every change to a child required a new version of its parent, the parent's
history would grow for changes that do not affect the parent itself. Over time,
this couples unrelated migrations and makes each parent schema responsible for
details of its children.

A nested version family gives that child an independent history. The parent
stores the child's version family, and migration composes automatically: when
loading, the parent reaches its latest schema and the nested value reaches its
current model. Saving selects the latest schema in both families. Each
migration can focus on the part of the data it owns.

This separation is especially useful when a child is shared by multiple parent
types. Its schema changes and migration rules can be defined once and reused,
while each parent keeps a history focused on its own fields. Applications still
work with the current child model directly, without carrying version wrappers
through business logic.

## More examples

The [generic containers](crates/thisversion/examples/generics.rs),
[fallible migrations](crates/thisversion/examples/fallible.rs), and
[convergent schemas](crates/thisversion/examples/converge.rs) examples cover
additional conversion patterns.

## FAQ

### Why the name?

`thisversion` takes inspiration from [`thiserror`](https://github.com/dtolnay/thiserror)
where both crates aim to make a focused part of Rust development easier to
express.
