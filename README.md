# lessvec

A minimal, educational Vec-like collection implemented with only the Rust standard library.

[![crates.io](https://img.shields.io/crates/v/lessvec.svg)](https://crates.io/crates/lessvec) [![docs.rs](https://img.shields.io/docsrs/lessvec/latest)](https://docs.rs/lessvec) [![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE) [![Build Status](https://github.com/bakayu/lessvec/actions/workflows/CI.yml/badge.svg)](https://github.com/bakayu/lessvec/actions/workflows/CI.yml) [![GitHub tag](https://img.shields.io/github/v/tag/bakayu/lessvec.svg)](https://github.com/bakayu/lessvec/releases)

## Quick example

```rust
use lessvec::LessVec;

let mut v = LessVec::new();
v.push(1);
v.push(2);
assert_eq!(v.as_slice(), &[1, 2]);
```

## Prelude & macro

`lessvec` exposes a small prelude with the `LessVec` type and the `lessvec!` macro:

```rust
use lessvec::prelude::*;

// macro to construct a LessVec (same syntax as std `vec!`)
let v = lessvec![1, 2, 3];
assert_eq!(v.as_slice(), &[1, 2, 3]);

// repeating element form (requires Clone)
let r = lessvec![5; 4];
assert_eq!(r.as_slice(), &[5, 5, 5, 5]);
```

You can also import the macro directly if you prefer:
```rust
use lessvec::lessvec;
let v = lessvec![1, 2, 3];
```

## Running the included example

The repository includes an example at `examples/basics.rs`. Run it with:

```bash
cargo run --example basics
```

You can run the test suite with:

```bash
cargo test
```

## License

[MIT LICENSE](./LICENSE)
