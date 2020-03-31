#ifndef _ENCLAVE_H
#define _ENCLAVE_H

#include <sgx_attributes.h>
#include <sgx_trts.h>
#include <sgx_tseal.h>

#include <stdint.h>
#include <stdlib.h>

/** Enclave sealing policy:
 *  sealing keys can be derived using MRENCLAVE or MRSIGNER. */
#define ENCLAVE_SEALING_POLICY SGX_KEYPOLICY_MRENCLAVE

/** Enclave flags that will matter for sealing/unsealing secrets (keys). */
#define ENCLAVE_SEALING_ATTRIBUTES (SGX_FLAGS_INITTED | SGX_FLAGS_DEBUG | SGX_FLAGS_MODE64BIT) 

int seal_keys(uint8_t** sealed_keys, size_t* sealed_size);

int unseal_keys(const uint8_t* sealed_data);

#endif
