# `pod-app`

The native app that _pod_ uses to interface with the `pod-enclave`. It implements [native messaging] and
therefore can be used from within a browser environment such as [`pod-ext`].

## Installation

Coming soon...
You can install the app together with the enclave file `pod_enclave.signed.so` using the provided simple
`install.sh` script. This script will install the `pod-app` into the local cargo registry, and will create
`$HOME/.local/share/pod-app` local storage dir and copy the enclave file into it. The sealed private key
file `private_key.sealed` will also reside there.

```
./install.sh
```

## Native messages handled by `pod-app`

All requests are in JSON format. See [native messaging] for more info.

* **Requests**:
  - `{ "msg" : "get_quote",  "spid": "01234567abcdef" }`
  - `{ "msg" : "sign_challenge", "challenge" : "AAADEADBEEF" }`

* **Responses**
  - `{ "msg" : "get_quote", "quote" : "AAAAA...AAA" }`
  - `{ "msg" : "sign_challenge", "signed" : "BBBBBAAAAABB" }`
  - `{ "msg" : "error", "description" : "description of the error" }`

## Development

### Building

Simply run from the repo's root:

```
cargo build
```

This will build [`pod-enclave`] by default which is a prerequisite for the `pod-app`.


## Examples

### `browser_mock` app

For testing purposes, there is a `browser_mock` app as part of the `pod-app` which can be used for
manual testing of the native messaging responses of the `pod-app`.

To run it, first make sure `pod-app` is built:

```
cargo build
```

And then run:

```
cargo run --example browser_mock target/debug/pod-app
```

You can send all request-type messages to the `pod-app` using `browser_mock` app and observe its
response. For instance:

```
> { "msg": "sign_challenge", "challenge": "deadbeef" }
{"msg":"sign_challenge","signed":"NYPXlzY98WUawum6yFQdelyzVoxC5VdguSSJ022ZJYyFc1W0DmZjnXP6t5t/gVwnckigP5u44yKmi7bIimiRBw=="}
```

[native messaging]: https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Native_messaging#Closing_the_native_app

### `webservice_client` app

For some end-to-end testing, you can use the provided simple test client which
exposes two bits of functionality: registering and authenticating with the [`pod-server`]
web service.

The former takes a username and Service Provider's ID (SPID):

```
cargo run --example webservice_client -- register johndoe deadbeef123456
```

This command will initiate a POST request to `/register` entrypoint.

The latter on the other hand takes only your username as an argument:

```
cargo run --example webservice_client -- authenticate johndoe
```

This command initiates 3 requests: a GET to `/auth` to obtain a challenge,
a POST to `/auth` to validate the challenge and authenticate with the
service, and finally a GET to `/` to verify that we've indeed successfully
signed in.

[`pod-server`]: https://github.com/golemfactory/proofofdevice/tree/master/pod-server
[`pod-enclave`]: https://github.com/golemfactory/proofofdevice/tree/master/pod-enclave
[native-messaging]: https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Native_messaging

