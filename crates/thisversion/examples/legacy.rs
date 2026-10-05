use std::time::Duration;

use serde::{Deserialize, Serialize};
use thisversion::traits::{IntoModel, IntoSchema};
use thisversion::{VersionBoundary, VersionFamily};

//============================================================================//
// V0                                                                         //
//============================================================================//

/// The legacy shape used by an existing codebase.
///
/// This data was serialized unwrapped, so the caller must know that it
/// represents V0 before deserializing it.
#[derive(Deserialize, Serialize)]
struct SettingsV0 {
    // Previous versions called this `label` or `title`. These aliases remain
    // here because this compatibility reader may need to accept all spellings
    // forever.
    #[serde(alias = "label", alias = "title")]
    name: String,

    // Previous versions did not have this, so old serializations should behave
    // as if the setting was absent.
    #[serde(default)]
    retries: Option<u8>,

    // Older files expressed the timeout in seconds. Newer files used
    // milliseconds. Exactly one should exist, depending on when the data was
    // serialized.
    #[serde(default)]
    timeout_secs: Option<u32>,
    #[serde(default)]
    timeout_ms: Option<u64>,

    // Earlier configurations called this `url`, and HTTPS was implied when
    // no scheme was present. Newer versions require the scheme be explicit.
    #[serde(alias = "url")]
    endpoint: String,
}

impl From<SettingsV0> for SettingsV1 {
    fn from(previous: SettingsV0) -> Self {
        fn normalize_endpoint(endpoint: String) -> String {
            if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
                endpoint
            } else {
                format!("https://{endpoint}")
            }
        }

        let timeout = match (previous.timeout_secs, previous.timeout_ms) {
            (Some(secs), None) => Duration::from_secs(u64::from(secs)),
            (None, Some(millis)) => Duration::from_millis(millis),
            (None, None) => Duration::from_secs(30),
            (Some(_), Some(_)) => {
                unreachable!("legacy settings never contained both timeout fields")
            }
        };

        Self {
            name: previous.name,
            retries: previous.retries.unwrap_or(0),
            timeout,
            endpoint: normalize_endpoint(previous.endpoint),
        }
    }
}

//============================================================================//
// VERSIONING                                                                 //
//============================================================================//

#[derive(Deserialize, Serialize, VersionFamily)]
#[serde(tag = "version", content = "data")]
enum SettingsVersions {
    V0(SettingsV0),
    V1(SettingsV1),
}

//============================================================================//
// V1                                                                         //
//============================================================================//

/// The latest persisted shape. It is also the bridge between persistence and
/// the canonical application model.
#[derive(Deserialize, Serialize, VersionBoundary)]
#[thisversion(model = Settings)]
struct SettingsV1 {
    name: String,
    retries: u8,
    #[serde(rename = "timeout_ms", with = "duration_millis")]
    timeout: Duration,
    endpoint: String,
}

/// Serde representation for the V1 timeout.
///
/// The application uses `Duration`, while the persisted schema uses an exact,
/// language-neutral integer count of milliseconds.
mod duration_millis {
    use super::Duration;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let millis = u64::try_from(duration.as_millis()).map_err(serde::ser::Error::custom)?;
        serializer.serialize_u64(millis)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        u64::deserialize(deserializer).map(Duration::from_millis)
    }
}

//============================================================================//
// APPLICATION                                                                //
//============================================================================//

/// The cleaned-up model used by the application after migration.
#[derive(Debug, PartialEq, Eq)]
struct Settings {
    name: String,
    retries: u8,
    timeout: Duration,
    endpoint: String,
}

fn main() {
    // Legacy data had no version wrapper. The application knows that this
    // payload is V0, so it chooses that snapshot explicitly.
    let settings = SettingsVersions::V0(
        serde_json::from_str::<SettingsV0>(
            r#"{
            "label": "worker",
            "timeout_secs": 15,
            "endpoint": "api.example.com"
        }"#,
        )
        .unwrap(),
    )
    .into_model();

    assert_eq!(
        settings,
        Settings {
            name: "worker".to_owned(),
            retries: 0,
            timeout: Duration::from_secs(15),
            endpoint: "https://api.example.com".to_owned(),
        }
    );

    // Once migrated, new persistence uses the explicit latest version.
    // Serializing the latest snapshot directly would omit the version wrapper.
    // In that case, record the schema version separately: today's latest
    // schema may no longer be the latest when the data is read again.
    let persisted: SettingsVersions = settings.into_schema();
    let persisted = serde_json::to_string(&persisted).unwrap();
    assert!(persisted.contains(r#""version":"V1""#));
    assert!(persisted.contains(r#""timeout_ms":15000"#));
}
