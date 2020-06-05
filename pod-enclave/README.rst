===========
``pod-enclave``
===========

This sample consists of the PoD SGX enclave (``pod_enclave``).

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
- (Newer versions of SGX PSW/SDK/SSL should work, 2.5 was chosen for Ubuntu 16.04 compatibility)

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

To build the enclave, and library, just run ``make``. To install the library and
the application, run ``sudo make install``.

