===========================
Proof of Device sample code
===========================

This sample consists of the PoD SGX enclave (``pod_enclave``), a shared library that exposes
the enclave functionality (``pod_library``) and an application that uses the library and enclave
to perform data signing (``pod_app``).

The enclave uses ED25519 keys for signing. Public key size is 32 bytes, signature size is 64 bytes.
See `<https://en.wikipedia.org/wiki/EdDSA#Ed25519>`_.
Enclave public key is stored in the ``report_data`` field of enclave SGX quote.

Requirements
============

- This sample was tested on Ubuntu 16.04 LTS, but should work on newer systems as well
- Intel SGX capable hardware
- SGX EPID driver installed and running, version 1.9 or above:
  `<https://github.com/intel/linux-sgx-driver>`_
- SGX platform software (PSW) and SDK version 2.5:
  `<https://download.01.org/intel-sgx/linux-2.5/>`_
- SGX SSL library version 2.5 with OpenSSL v1.1.1d:
  `<https://github.com/intel/intel-sgx-ssl/tree/lin_2.5_1.1.1d>`_
  `<https://www.openssl.org/source/old/1.1.1/openssl-1.1.1d.tar.gz>`_
- (Never versions of SGX PSW/SDK/SSL should work, 2.5 was chosen for Ubuntu 16.04 compatibility)

Building
========

Check Makefiles for possible configuration options, mainly paths to SGX SDK/SSL (``SGX_SDK``,
``SGX_SSL``). Default values match default installation paths of these components
(in ``/opt/intel``).

You need an enclave signing key, which is a 3072-bit RSA key with exponent ``3``. You can generate
it with ``openssl``::

   openssl genrsa -out enclave_signing_key.pem -3 3072

``ENCLAVE_SIGNING_KEY`` environment variable/Makefile option should contain path to the enclave
signing key.

To build the enclave, library and application, just run ``make``. To install the library and
the application, run ``sudo make install``.

Running
=======

You need to register on Intel Attestation Service (IAS) to get an EPID Service Provider ID:
`<https://api.portal.trustedservices.intel.com/EPID-attestation>`_.

Commands below assume that the build succeeded and that current directory is the main directory of
the repository (the one containing this README).

Initialization
--------------

To generate a new enclave key pair and export enclave quote, run::

   $ pod_app/pod_app init -e pod_enclave/pod_enclave.signed.so -i $IAS_SPID -t $IAS_QUOTE_TYPE

``IAS_SPID`` and ``IAS_QUOTE_TYPE`` should be set to values obtained during IAS registration.

::

   Loading enclave from file 'pod_enclave/pod_enclave.signed.so'
   Enclave loaded successfully, id = 0x2
   Enclave initializing...
   Generating enclave private key...
   Sealing enclave keys...
   Saving sealed enclave state to 'pod_data.sealed'
   Enclave public key: 07a0597bfc6942f4ff01a3913cdc705cbbac232e182513aae0f1d4732807dfa9
   Copying enclave public key...
   Enclave initialization OK
   Saving public enclave key to 'pod_pubkey'
   Enclave quote saved to 'pod.quote'
   Enclave unloaded

Signing
-------

After the enclave keys are generated, you can start signing data (``pod_enclave/pod_enclave.c``
in this example)::

   $ pod_app/pod_app sign -e pod_enclave/pod_enclave.signed.so -D pod_enclave/pod_enclave.c -S signature
   Loading enclave from file 'pod_enclave/pod_enclave.signed.so'
   Enclave loaded successfully, id = 0x2
   Loading sealed enclave state from 'pod_data.sealed'
   Enclave initializing...
   Unsealing enclave keys...
   Enclave public key: 07a0597bfc6942f4ff01a3913cdc705cbbac232e182513aae0f1d4732807dfa9
   Enclave initialization OK
   Signed 9083 bytes of data
   Saved signature to 'signature'
   Enclave unloaded

A simple Python script is provided for signature verification since OpenSSL command line lacks the
ability to import raw ED25519 keys. The ``ed25519`` package is required (``pip3 install ed25519``)::

   $ python3 ed_verify.py pod_pubkey signature pod_enclave/pod_enclave.c
   Public key: 07a0597bfc6942f4ff01a3913cdc705cbbac232e182513aae0f1d4732807dfa9
   Signature verified OK.
