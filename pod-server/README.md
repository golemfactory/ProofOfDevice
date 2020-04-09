# pod-server

A simple implementation of a web service facilitating the Proof-of-device
at the service provider's end.

## Basic usage

The minimal config file to run the server consists of the IAS valid API key:

```toml
api_key = "0123456abcdef"
```

By default, the server binds itself to `127.0.0.1:8088` address. You can tweak
it by appending a `[server]` section to the config file:

```toml
api_key = "0123456abcdef"

[server]
address = "127.0.0.1"
port = 8088
```

Finally, when invoking the server from the command line, at present, you are
required to specify the path to the config file:

```
cargo run -- config.toml
```

## Testing with simple test client

For some end-to-end testing, you can use the provided simple test client which
allows you to send a quote and nonce to the server to the `/register`
entrypoint and await a reply.

You can invoke the test client as follows:

```
cargo run --example test_client -- <path_to_quote> --nonce "deadbeef123456"
```
