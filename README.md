# lessvec

A minimal, educational Vec-like collection implemented with only the Rust standard library.

[![crates.io](https://img.shields.io/crates/v/lessvec.svg)](https://crates.io/crates/lessvec) [![docs.rs](https://docs.rs/lessvec/badge.svg)](https://docs.rs/lessvec) [![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE) [![Build Status](https://github.com/OWNER/REPO/actions/workflows/CI.yml/badge.svg)](https://github.com/OWNER/REPO/actions/workflows/CI.yml) [![GitHub tag](https://img.shields.io/github/v/tag/OWNER/REPO.svg)](https://github.com/OWNER/REPO/releases)

## Quick example

```rust
use lessvec::LessVec;

let mut v = LessVec::new();
v.push(1);
v.push(2);
assert_eq!(v.as_slice(), &[1, 2]);
```

## License

[MIT LICENSE](./LICENSE)
