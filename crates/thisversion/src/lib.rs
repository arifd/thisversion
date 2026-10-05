#![no_std]
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

// Let generated absolute paths work inside this crate as well as downstream.
#[allow(unused_extern_crates)]
extern crate self as thisversion;

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod traits;

#[cfg(feature = "serde")]
pub mod serde;

pub use thisversion_derive::{VersionBoundary, VersionFamily};
