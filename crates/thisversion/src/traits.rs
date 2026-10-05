//! Traits for converting between versioned schemas and application models.
//!
//! A schema type represents data in a persisted format; an application model
//! is the representation the program uses. Implement [`IntoSchema`] to choose
//! a schema representation when writing data, and implement [`IntoModel`] or
//! [`TryIntoModel`] to read schema data into the application model.
//!
//! These traits consume their input so conversions can move owned fields
//! without cloning. Container implementations for `Option` (and, with the
//! `alloc` feature, `Vec`) apply the same conversion to each contained value.
//! The derive macros can generate the schema and version-family conversions
//! for common cases.

//============================================================================//
// INTO SCHEMA                                                                //
//============================================================================//

/// Converts an application model into a selected schema representation.
///
/// The result is a data value in the requested representation. It may be a bare
/// schema value, a version-family value, or a container of schema values. A
/// model may support multiple target representations.
///
/// The `VersionBoundary` derive constructs the latest schema value, then uses
/// `From` to convert it into `S`. Requesting its version family selects the
/// latest variant; requesting an older schema requires an explicit conversion
/// from the latest schema. Such conversions may discard information, so a round
/// trip through a schema need not preserve the original model.
pub trait IntoSchema<S>: Sized {
    /// Converts this model into the requested schema representation.
    fn into_schema(self) -> S;
}

impl<T, S> IntoSchema<Option<S>> for Option<T>
where
    T: IntoSchema<S>,
{
    fn into_schema(self) -> Option<S> {
        self.map(IntoSchema::into_schema)
    }
}

#[cfg(feature = "alloc")]
impl<T, S> IntoSchema<alloc::vec::Vec<S>> for alloc::vec::Vec<T>
where
    T: IntoSchema<S>,
{
    fn into_schema(self) -> alloc::vec::Vec<S> {
        self.into_iter().map(IntoSchema::into_schema).collect()
    }
}

//============================================================================//
// INTO MODEL                                                                 //
//============================================================================//

/// Converts a schema value into its application model without a recoverable
/// conversion error.
///
/// Implement this when every value of the schema type can be represented by the
/// model. Use [`TryIntoModel`] when conversion can reject a value.
pub trait IntoModel: Sized {
    /// The application model produced from this schema type.
    type Model;

    /// Converts this schema value into its application model.
    fn into_model(self) -> Self::Model;
}

impl<T: IntoModel> IntoModel for Option<T> {
    type Model = Option<T::Model>;

    fn into_model(self) -> Self::Model {
        self.map(IntoModel::into_model)
    }
}

#[cfg(feature = "alloc")]
impl<T: IntoModel> IntoModel for alloc::vec::Vec<T> {
    type Model = alloc::vec::Vec<T::Model>;

    fn into_model(self) -> Self::Model {
        self.into_iter().map(IntoModel::into_model).collect()
    }
}

//============================================================================//
// TRY INTO MODEL                                                             //
//============================================================================//

/// Converts a schema, version family, or container into an application model,
/// allowing conversion to fail.
///
/// Fallible latest schemas choose the final error type. Fallible version-family
/// derives use that same type while migrating and converting the latest schema.
/// Recursive fields propagate their errors in the same way.
///
/// Deriving `VersionBoundary` without an error type also supplies
/// `TryIntoModel` with [`core::convert::Infallible`] as its error, so both
/// model conversion entry points remain available to adapters.
pub trait TryIntoModel: Sized {
    /// The application model produced from this schema type.
    type Model;

    /// The error returned when a schema value cannot be represented by the
    /// application model.
    type Error;

    /// Converts this schema value into its application model.
    fn try_into_model(self) -> Result<Self::Model, Self::Error>;
}

impl<T: TryIntoModel> TryIntoModel for Option<T> {
    type Model = Option<T::Model>;
    type Error = T::Error;

    fn try_into_model(self) -> Result<Self::Model, Self::Error> {
        self.map(TryIntoModel::try_into_model).transpose()
    }
}

#[cfg(feature = "alloc")]
impl<T: TryIntoModel> TryIntoModel for alloc::vec::Vec<T> {
    type Model = alloc::vec::Vec<T::Model>;
    type Error = T::Error;

    fn try_into_model(self) -> Result<Self::Model, Self::Error> {
        self.into_iter().map(TryIntoModel::try_into_model).collect()
    }
}
