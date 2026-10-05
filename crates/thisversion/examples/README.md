# Examples

## Basic examples

These examples introduce the core model behind the library and are a good place to start.

| Example                  | Description                                                                        |
| ------------------------ | ---------------------------------------------------------------------------------- |
| [`basic.rs`](basic.rs)   | Demonstrates a common use case using derive macros to reduce boilerplate.          |
| [`manual.rs`](manual.rs) | Demonstrates the same basic example by manually implementing the required traits.  |
| [`serde.rs`](serde.rs)   | Demonstrates the same basic example with Serde integration for persistence.        |

## Advanced examples

These examples demonstrate more advanced use cases and composition patterns supported by the library.

| Example                      | Description                                                                       |
| ---------------------------- | --------------------------------------------------------------------------------- |
| [`generics.rs`](generics.rs) | Demonstrates versioning types that use generics.                                  |
| [`converge.rs`](converge.rs) | Demonstrates multiple migration chains converging into a shared application type. |
| [`nested.rs`](nested.rs)     | Demonstrates composition of nested types with independent version histories.        |
| [`fallible.rs`](fallible.rs) | Demonstrates fallibility in a nested context and how errors can compose.          |
| [`legacy.rs`](legacy.rs)     | Demonstrates adopting the library into an existing codebase.                      |
| [`pinned.rs`](pinned.rs)     | Demonstrates pinning a parent to a child snapshot, with explicit downgrade.      |
