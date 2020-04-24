# Proof-of-device

Proof-of-device, or _pod_, is another take at 2FA or rather U2F. Here, however, the burden of storing
keys for signing and proving your identity is managed by the SGX enclave. The service you're
authenticating with sends you challenge which you sign using a private key embedded within the
enclave in your Intel CPU. The system is very secure since not even you have the knowledge of
the private key that's stored within the enclave.

## Project structure

The project comprises of four main components:
* [`pod-enclave`] -- This is where the private key used for signing authentication challenge requests
                     is generated and then stored. Note that the private key is actually stored on the
                     host, however, in an enclave-sealed form which only the enclave that generated it
                     can unseal to then use it for signing.
* [`pod-app`] -- This is the native app that _pod_ uses to interface with the `pod-enclave`. It implements
                 [native messaging] and therefore can be used from within a browser environment.
* [`pod-browser`] -- This is the browser extension _pod_ uses as a GUI for the enduser of the _pod_
                     authentication mechanism.
* [`pod-server`] -- This is the web server that the service provider who offers _pod_ as an added authentication
                    mechanism uses.

[`pod-enclave`]: https://github.com/golemfactory/ProofOfDevice/tree/master/crates/c-api/pod-enclave
[`pod-app`]: #pod-app
[`pod-browser`]: https://github.com/golemfactory/ProofOfDevice
[`pod-server`]: https://github.com/golemfactory/ProofOfDevice/tree/master/crates/server

For each of the components, follow the links to learn more and check out how to build and run them.

## `pod-app`

The native app that _pod_ uses to interface with the `pod-enclave`. It implements [native messaging] and
therefore can be used from within a browser environment such as [`pod-browser`].

### Native messages handled by `pod-app`

All requests are in JSON format. See [native messaging] for more info.

* **Requests**:
  - `{ "msg" : "get_quote",  "spid": "01234567abcdef" }`
  - `{ "msg" : "sign_challenge", "challenge" : "AAADEADBEEF" }`

* **Responses**
  - `{ "msg" : "get_quote", "quote" : "AAAAA...AAA" }`
  - `{ "msg" : "sign_challenge", "signed" : "BBBBBAAAAABB" }`
  - `{ "msg" : "error", "description" : "description of the error" }`

### Building

Simply run from the repo's root:

```
cargo build
```

### Testing with `browser_mock` app

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

## Caveats

This project currently builds and was tested on Linux only (both Ubuntu 18.04 and Arch). In the future, it is envisaged
to support Windows however.

