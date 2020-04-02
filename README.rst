===========================
Proof of Device sample code
===========================

This sample consists of the PoD SGX enclave (``pod_enclave``) and an application that uses
the enclave to perform data signing (``pod_app``).

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

To build both the enclave and application, just run ``make``.

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
   Generating enclave key...
   Sealing keys...
   Saving sealed enclave state to 'pod_data.sealed'
   Saving public enclave key to 'pod_pubkey.pem'
   Public enclave key hash: 36e24b93edb3acd51112db95225b2c32c0331c6d69e9da290b2235e5495ba16c
   Enclave initialization OK
   Enclave quote saved to 'pod.quote'

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
   Public enclave key hash: 36e24b93edb3acd51112db95225b2c32c0331c6d69e9da290b2235e5495ba16c
   Enclave initialization OK
   Signature size: 512 bytes
   Signed 12752 bytes of data
   Saved signature to 'signature'

   $ openssl dgst -sha256 -verify pod_pubkey.pem -signature signature pod_enclave/pod_enclave.c
   Verified OK
