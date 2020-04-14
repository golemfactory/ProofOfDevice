#ifndef POD_ENCLAVE_H
#define POD_ENCLAVE_H

#include <sgx_attributes.h>
#include <sgx_trts.h>
#include <sgx_tseal.h>

#include <stdint.h>
#include <stdlib.h>

/*! Enclave sealing policy:
 *  sealing keys can be derived using MRENCLAVE or MRSIGNER. */
#define ENCLAVE_SEALING_POLICY SGX_KEYPOLICY_MRENCLAVE

/*! Enclave flags that will matter for sealing/unsealing secrets (keys). */
#define ENCLAVE_SEALING_ATTRIBUTES (SGX_FLAGS_INITTED | SGX_FLAGS_DEBUG | SGX_FLAGS_MODE64BIT) 

/*! Size of the EC public key (in bytes). */
#define EC_PUBLIC_KEY_SIZE 32

/*! Size of the EC signature (in bytes). */
#define EC_SIGNATURE_SIZE 64

/*! EC curve ID used for digital signatures. */
#define EC_CURVE_ID EVP_PKEY_ED25519

int seal_keys(void);

int unseal_keys(const uint8_t* sealed_data, size_t sealed_size);

#endif
