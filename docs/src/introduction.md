<div align="center">

<img alt="thisversion logo" src="https://raw.githubusercontent.com/arifd/thisversion/main/docs/assets/thisversion.svg" width="256" height="256">

<h1>thisversion</h1>

**Keep schema history out of your application model.**

[<img alt="crates.io" src="https://img.shields.io/crates/v/thisversion.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/thisversion)
[<img alt="MSRV 1.88+" src="https://img.shields.io/badge/MSRV-1.88%2B-8da0cb?style=for-the-badge&labelColor=555555" height="20">](https://www.rust-lang.org/)
[<img alt="no_std supported" src="https://img.shields.io/badge/no__std-supported-35b99b?style=for-the-badge&labelColor=555555" height="20">](https://github.com/arifd/thisversion#persistence)
[<img alt="unsafe forbidden" src="https://img.shields.io/badge/unsafe-forbidden-e9ae35?style=for-the-badge&labelColor=555555" height="20">](https://github.com/arifd/thisversion/blob/main/Cargo.toml)

<a href="https://docs.rs/thisversion">API reference</a> ·
<a href="https://github.com/arifd/thisversion/tree/main/crates/thisversion/examples">Examples</a> ·
<a href="https://github.com/arifd/thisversion/blob/main/CHANGELOG.md">Changelog</a>

</div>

# Introduction

This guide explains **how** to use `thisversion`, with a particular focus on
advanced workflows and the decisions involved in evolving persisted data.

For the **what**, the available derives, traits, types and functions, see the
[API reference](https://docs.rs/thisversion).

The chapters below focus on decisions that arise as your data evolves. They can
be read independently:

- **[Adding the next schema version](./next-version.md)**
  Evolve a working schema while preserving the meaning of existing data.

- **[Adopting existing persisted data](./legacy.md)**
  Bring unversioned data into an explicit version history.

- **[Choosing how nested schemas evolve](./nesting.md)**
  Decide whether nested schemas should have independent histories or use a
  fixed representation.

- **[Fallible migrations](./failures.md)**
  Add migrations that can reject values and return errors.

- **[Manually implementing conversions](./manual.md)**
  Map schemas to models when the derive's field mapping is not enough.
