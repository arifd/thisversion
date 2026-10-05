//! Serde adapters for reading historical schemas and writing schema values.
//!
//! Enable the crate's `serde` feature to use these helpers. Deserialization
//! first lets Serde build the requested schema type, then migrates it to its
//! latest version and converts it into the associated application model.
//! Serialization converts a model into a chosen schema representation before
//! asking Serde to write it.
//!
//! Use [`deserialize_into_model`] on a field in a type that derives
//! [`Deserialize`](::serde::Deserialize). For a complete value, deserialize
//! the schema and convert it with [`TryIntoModel::try_into_model`]. Use
//! [`serialize_into_schema`] for a field in a type that derives
//! [`Serialize`](::serde::Serialize). When you
//! own the model value, [`IntoSchema::into_schema`] avoids the clone required
//! by the borrowed serialization helper.
//!
//! Conversion errors must implement [`Display`](core::fmt::Display) so they
//! can be reported as Serde errors. This requirement applies only to these
//! adapters; the conversion traits themselves do not require displayable
//! errors.

use crate::traits::{IntoSchema, TryIntoModel};

/// Deserializes a schema value and converts it into its associated model.
///
/// Use this with Serde's `deserialize_with` attribute when a containing type
/// derives `Deserialize` normally.
///
/// `T` is the schema type decoded by Serde. Its value is migrated to its latest
/// representation before conversion to `T::Model`.
///
/// # Example
///
/// ```rust
/// # use thisversion::{VersionBoundary};
/// #
/// # #[derive(Deserialize, Serialize)]
/// # struct UserV1 { name: String }
/// #
/// # #[derive(Deserialize, Serialize, VersionBoundary)]
/// # #[thisversion(model = User)]
/// # struct UserV2 { name: String }
/// #
/// # impl From<UserV1> for UserV2 {
/// #     fn from(previous: UserV1) -> Self {
/// #         Self { name: previous.name }
/// #     }
/// # }
/// #
/// #
/// use serde::{Deserialize, Serialize};
/// use thisversion::serde::deserialize_into_model;
/// use thisversion::VersionFamily;
///
/// #[derive(Deserialize, Serialize, VersionFamily)]
/// #[serde(tag = "version", content = "data")]
/// enum UserVersions {
///     V1(UserV1),
///     V2(UserV2),
///     //...
/// }
///
/// // ...
///
/// #[derive(Debug, PartialEq, Eq)]
/// struct User { name: String }
///
/// #[derive(Deserialize)]
/// struct Record {
///     #[serde(deserialize_with = "deserialize_into_model::<UserVersions, _>")]
///     user: User,
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let record: Record = serde_json::from_str(
///     r#"{"user":{"version":"V1","data":{"name":"Ada"}}}"#,
/// )?;
///
/// assert_eq!(record.user.name, "Ada");
///
/// # Ok(())
/// # }
/// ```
pub fn deserialize_into_model<'de, T, D>(deserializer: D) -> Result<T::Model, D::Error>
where
    T: ::serde::Deserialize<'de> + TryIntoModel,
    <T as TryIntoModel>::Error: core::fmt::Display,
    D: ::serde::Deserializer<'de>,
{
    <T as ::serde::Deserialize>::deserialize(deserializer)?
        .try_into_model()
        .map_err(::serde::de::Error::custom)
}

/// Converts a model into the requested schema representation and serializes it.
///
/// `T` selects the schema representation, including a bare schema or version
/// family. Conversion follows its `IntoSchema<T>` implementation.
///
/// Use this with Serde's `serialize_with` attribute when a containing type
/// derives `Serialize` normally.
///
/// `T::Model` must implement `Clone` because Serde passes fields to
/// `serialize_with` by shared reference, while `IntoSchema` intentionally
/// consumes the model. The model is therefore cloned before conversion. When
/// ownership is available, prefer calling `IntoSchema::into_schema` directly to
/// avoid the clone.
///
/// # Example
///
/// ```rust
/// # use thisversion::{VersionBoundary};
/// #
/// # #[derive(Deserialize, Serialize)]
/// # struct UserV1 { name: String }
/// #
/// # #[derive(Deserialize, Serialize, VersionBoundary)]
/// # #[thisversion(model = User)]
/// # struct UserV2 { name: String }
/// #
/// # impl From<UserV1> for UserV2 {
/// #     fn from(previous: UserV1) -> Self {
/// #         Self { name: previous.name }
/// #     }
/// # }
/// #
/// #
/// use serde::{Deserialize, Serialize};
/// use thisversion::serde::serialize_into_schema;
/// use thisversion::VersionFamily;
///
/// #[derive(Deserialize, Serialize, VersionFamily)]
/// #[serde(tag = "version", content = "data")]
/// enum UserVersions {
///     V1(UserV1),
///     V2(UserV2),
///     //...
/// }
///
/// // ...
///
/// #[derive(Debug, PartialEq, Eq, Clone)]
/// struct User { name: String }
///
/// #[derive(Serialize)]
/// struct Record {
///     #[serde(serialize_with = "serialize_into_schema::<UserVersions, _>")]
///     user: User,
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let record = Record { user: User { name: "Ada".into() } };
///
/// let json = serde_json::to_string(&record)?;
///
/// assert_eq!( json, r#"{"user":{"version":"V2","data":{"name":"Ada"}}}"# );
///
/// # Ok(())
/// # }
/// ```
pub fn serialize_into_schema<T, S>(value: &T::Model, serializer: S) -> Result<S::Ok, S::Error>
where
    T: ::serde::Serialize + TryIntoModel,
    T::Model: Clone + IntoSchema<T>,
    S: ::serde::Serializer,
{
    IntoSchema::<T>::into_schema(value.clone()).serialize(serializer)
}
