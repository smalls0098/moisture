# moisture
**Moisture** is a Rust syntax parsing library intended for use with procedural macros. It is based on [syn](https://crates.io/crates/syn), [quote](https://crates.io/crates/quote) and [proc-macro2](https://crates.io/crates/proc_macro2). It's primary function is to crawl the syntax tree provided by the [syn](https://crates.io/crates/syn), process the individual items provided via registered callbacks handling the items, then return the token stream to the given objects. **Moisture** is intended to be a procedural macro solution for complex handling of the Rust syntax tree.

The changelog can be found [here](https://github.com/frank2/moisture/blob/main/CHANGELOG.md) and further documentation can be found [here](https://docs.rs/moisture).
