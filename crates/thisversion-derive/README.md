<div align="center">

<img alt="thisversion logo" src="https://raw.githubusercontent.com/arifd/thisversion/main/docs/assets/thisversion.svg" width="192" height="192">

<h1>Thisversion-derive</h1>

**Keep schema history out of your application model.**

[<img alt="crates.io" src="https://img.shields.io/crates/v/thisversion-derive.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/thisversion-derive)
[<img alt="MSRV 1.88+" src="https://img.shields.io/badge/MSRV-1.88%2B-8da0cb?style=for-the-badge&labelColor=555555" height="20">](https://www.rust-lang.org/)

<a href="https://crates.io/crates/thisversion">Main crate</a> ·
<a href="https://docs.rs/thisversion-derive">API documentation</a> ·
<a href="https://github.com/arifd/thisversion">Source repository</a>

</div>

## tbisversion-derive

`thisversion-derive` implements the `VersionFamily` and `VersionBoundary`
derive macros used by [`thisversion`](https://crates.io/crates/thisversion).
They connect historical schema types to an application's current model while
keeping migrations explicit and checked by Rust's type system.

Most applications should depend on `thisversion`, which re-exports both
derives, rather than depending on this implementation crate directly:

```toml
[dependencies]
thisversion = "0.1"
```

See the [main crate README](https://github.com/arifd/thisversion) for
usage, examples, and the project guide.
