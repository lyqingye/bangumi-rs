# anitomy-rs

A Rust port of [anitomy](https://github.com/erengy/anitomy/). Most of the credit goes to the original author, [erengy](https://github.com/erengy/).

This port was made to facilitate usage in Rust programs and also to support compiling under WASM. As such, parts of it have been redesigned to accommodate differences in idiomatic programming style.

## Differences

This is based off of the (as of writing) incomplete v2 rewrite of anitomy.

That being said, there are some differences from that version of anitomy:

- Parsing is zero copy to the extent possible:
    - Very few allocations are done outside of a few vecs to hold the state
    - Some allocations are done when they're forced to due to Rust safety constraints (e.g. merging contiguous blocks of memory)
    - Other allocations due to concatenating strings such as for release titles, episode titles, groups, etc.
- Lookup tables are done using the [PFH crate](https://github.com/rust-phf/rust-phf) instead of dynamically allocated maps
    - As a consequence, some keyword detection had to be removed.
- A few more extensions were added to facilitate a wider use case:
    - Subtitle formats (.ass, .ssa, .srt)
    - Archive formats (.zip, .7z)
    - More language detection (CHS, CHT, JPN, etc.)
- Some tests that were failing are now passing

As a result this **does not** aim to be fully compatible with upstream anitomy but it tries its best to be.

## Features

By default, no features are enabled. The following features can be enabled:

- **serde**: Adds support for `serde` (de)serialization.
- **wasm**: Adds support for exporting a `parse` function via WASM. This is essentially a `parse` function that takes the input and an `Options`.

### Compiling for WASM

This library is capable of being exported to a `.wasm` file using `wasm_bindgen`. It exports the following types:

- `ElementKind`
- `Element`
- `Options`
- `parse`

Except augmented to support `wasm_bindgen`. Using [`wasm-pack`](https://rustwasm.github.io/wasm-pack/) you can do the following:

```
wasm-pack build --target web --release --features wasm
```

## License

MPLv2
