# `pod-app`

The native app that _pod_ uses to interface with the `pod-enclave`. It implements [native messaging] and
therefore can be used from within a browser environment such as [`pod-browser`].

## Native messages handled by `pod-app`

All requests are in JSON format. See [native messaging] for more info.

* **Requests**:
  - `{ "msg" : "get_quote",  "spid": "01234567abcdef" }`
  - `{ "msg" : "sign_challenge", "challenge" : "AAADEADBEEF" }`

* **Responses**
  - `{ "msg" : "get_quote", "quote" : "AAAAA...AAA" }`
  - `{ "msg" : "sign_challenge", "signed" : "BBBBBAAAAABB" }`
  - `{ "msg" : "error", "description" : "description of the error" }`

## Building

Simply run from the repo's root:

```
cargo build
```

## Testing with `browser_mock` app

For testing purposes, there is a `browser_mock` app as part of the `pod-app` which can be used for
manual testing of the native messaging responses of the `pod-app`.

To run it, first make sure `pod-app` is built:

```
cargo build
```

And then run:

```
cargo run --bin browser_mock target/debug/pod-app
```

You can send all request-type messages to the `pod-app` using `browser_mock` app and observe its
response. For instance:

```
> { "msg": "sign_challenge", "challenge": "deadbeef" }
{"msg":"sign_challenge","signed":"NYPXlzY98WUawum6yFQdelyzVoxC5VdguSSJ022ZJYyFc1W0DmZjnXP6t5t/gVwnckigP5u44yKmi7bIimiRBw=="}
```

[native messaging]: https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Native_messaging#Closing_the_native_app

