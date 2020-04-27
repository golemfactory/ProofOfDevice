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
* [`pod-ext`] -- This is the browser extension connects _pod_ to generate quote and sign the challenge for the _pod-web_ authentication mechanism.
* [`pod-web`] -- This is the web app _pod-ext_ connects as a GUI for the end user of the _pod_.
* [`pod-server`] -- This is the web server that the service provider who offers _pod_ as an added authentication
                    mechanism uses.

[`pod-enclave`]: https://github.com/golemfactory/ProofOfDevice/tree/master/pod-enclave
[`pod-app`]: https://github.com/golemfactory/ProofOfDevice/tree/master/pod-app
[`pod-ext`]: https://github.com/golemfactory/ProofOfDevice/tree/master/pod-ext
[`pod-web`]: https://github.com/golemfactory/ProofOfDevice/tree/master/pod-web
[`pod-server`]: https://github.com/golemfactory/ProofOfDevice/tree/master/pod-server
[native messaging]: https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Native_messaging

For each of the components, follow the links to learn more and check out how to build and run them.

## Caveats

This project currently builds and was tested on Linux only (both Ubuntu 18.04 and Arch). In the future, it is envisaged
to support Windows however.

