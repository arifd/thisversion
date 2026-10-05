#![doc(
    html_logo_url = "https://raw.githubusercontent.com/arifd/thisversion/main/docs/assets/thisversion.svg",
    html_favicon_url = "https://raw.githubusercontent.com/arifd/thisversion/main/docs/assets/thisversion.svg"
)]
// Cargo relocates the README when packaging; use the manifest's current path.
#![doc = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/",
    env!("CARGO_PKG_README")
))]

mod boundary;
mod family;

use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::Span;
use quote::quote;

/// Resolves the runtime crate path, including Cargo aliases.
pub(crate) fn runtime() -> syn::Result<syn::Path> {
    match crate_name("thisversion").map_err(|_| {
        syn::Error::new(
            Span::call_site(),
            "could not find the `thisversion` runtime crate; add `thisversion` as a dependency",
        )
    })? {
        FoundCrate::Itself => Ok(syn::parse_quote!(::thisversion)),
        FoundCrate::Name(name) => {
            let ident = syn::Ident::new(&name, Span::call_site());
            Ok(syn::parse2(quote!(::#ident))?)
        }
    }
}

/// Connects a chronological enum through caller-provided `From` or `TryFrom`
/// conversions. Its latest schema defines the model and error types.
///
/// Generates `From<Schema>` for every family variant and model conversion
/// implementations. Infallible migrations and model conversion are the default.
/// Add `#[thisversion(fallible)]` to the enum when a migration or the latest
/// schema's model conversion can fail; migration errors must convert into its
/// model error.
#[proc_macro_derive(VersionFamily, attributes(thisversion))]
pub fn derive_version_family(input: TokenStream) -> TokenStream {
    syn::parse(input)
        .and_then(family::parse)
        .and_then(|family| Ok((family, runtime()?)))
        .map(|(family, runtime)| family::expand(family, &runtime))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Connects a family's latest schema version to its application model.
///
/// Apply this derive to the final variant's payload in a `VersionFamily`. The
/// family determines version order; this derive generates the model conversions
/// and does not check family membership or position.
///
/// Requires `#[thisversion(model = Model)]` and matching field names. With no
/// `error` option, generates `IntoModel`, its
/// `TryIntoModel<Error = Infallible>` adapter, and the model's `IntoSchema<S>`.
/// With `#[thisversion(model = Model, error = E)]`, generates fallible
/// `TryIntoModel` and `IntoSchema<S>`.
///
/// Model -> schema conversion constructs this schema's value, then converts it
/// into the requested `S` through `From`.
///
/// Mark a field with `#[thisversion(nested)]` to recursively convert a family
/// wrapper in both directions. A bare schema value can name its family with
/// `#[thisversion(nested = Family)]`; loading wraps it in that family and
/// migrates it forward. Saving back to the pinned schema is available only when
/// the child model implements `IntoSchema<S>`.
#[proc_macro_derive(VersionBoundary, attributes(thisversion))]
pub fn derive_version_boundary(input: TokenStream) -> TokenStream {
    syn::parse(input)
        .and_then(boundary::parse::parse)
        .and_then(|boundary| Ok((boundary, runtime()?)))
        .map(|(boundary, runtime)| boundary::expand::expand(boundary, &runtime))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
