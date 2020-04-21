# Proof-of-device

Proof-of-device, or pod, is another take at 2FA or rather U2F. Here, however, the burden of storing
keys for signing and proving your identity is managed by the SGX enclave. The service you're
authenticating with sends you challenge which you sign using a private key embedded within the
enclave in your Intel CPU. The system is very secure since not even you have the knowledge of
the private key that's stored within the enclave.

## Table of contents

This repo is currently organised into two parts:
1. [pod-client] -- contains the sources to build the enclave, generate Intel Attestation Services
                   quote, and sign fed buffer/file.
2. [pod-server] -- contains the sources of the web service, or the service provider, that the
                   `pod-client` would like to authenticate with.

[pod-client]: https://github.com/golemfactory/ProofOfDevice/tree/master/pod-client
[pod-server]: https://github.com/golemfactory/ProofOfDevice/tree/master/pod-server

## License

We're working on it...

