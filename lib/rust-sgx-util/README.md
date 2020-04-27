# `rust-sgx-util`

A safe wrapper around Graphene's [`sgx_util`] C-library.

[`sgx_util`]: https://github.com/oscarlab/graphene/tree/master/Pal/src/host/Linux-SGX/tools

```toml
rust-sgx-util = "0.2"
```

For `serde` support, you can enable it with `with_serde` feature:

```toml
rust-sgx-util = { version = "0.2", features = ["with_serde"] }
```

## Prerequisites

Currently, this crate requires you compile and install `sgx_util` as
a shared library.

## Usage examples

You can find usage examples in the `examples` dir of the crate.
