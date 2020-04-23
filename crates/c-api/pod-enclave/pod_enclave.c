#include <assert.h>
#include <openssl/evp.h>
#include <sgx_utils.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>
#include <tSgxSSL_api.h>

#include "pod_enclave.h"
#include "pod_enclave_t.h"
#include "pod_enclave_mrsigner.h"

bool g_initialized = false;
EVP_PKEY_CTX* g_ec_ctx = NULL;
EVP_PKEY* g_private_key = NULL;
uint8_t g_public_key[EC_PUBLIC_KEY_SIZE] = {0};

/*! Enclave flags that will matter for sealing/unsealing secrets (keys).
 *  The second field (xfrm) is set to 0 as per recommendation in the
 *  Intel SGX Developer Guide, Sealing and Unsealing Process section.
 */
const sgx_attributes_t g_seal_attributes = {ENCLAVE_SEALING_ATTRIBUTES, 0};

void zero_memory(void* mem, size_t size) {
    memset_s(mem, size, 0, size);
}

#define PRINT_BUFFER_MAX 4096
void eprintf(const char* fmt, ...) {
	char buf[PRINT_BUFFER_MAX];
	buf[PRINT_BUFFER_MAX - 1] = 0;
	va_list ap;
	va_start(ap, fmt);
	vsnprintf(buf, PRINT_BUFFER_MAX, fmt, ap);
	va_end(ap);
	o_print(buf);
}

void _hexdump(void* data, size_t size) {
    uint8_t* ptr = (uint8_t*)data;

    for (size_t i = 0; i < size; i++)
        eprintf("%02x", ptr[i]);
    eprintf("\n");
}

#define hexdump(x) _hexdump((void*)&x, sizeof(x))

int generate_public_key(void) {
    int ret = -1;

    if (!g_ec_ctx)
        goto out;

    // export raw public key
    size_t pubkey_size;
    int openssl_ret = EVP_PKEY_get_raw_public_key(g_private_key, NULL, &pubkey_size);
    if (openssl_ret <= 0) {
        eprintf("Failed to get public key size: %d\n", openssl_ret);
        goto out;
    }

    if (pubkey_size != EC_PUBLIC_KEY_SIZE) {
        eprintf("Invalid public key size\n");
        goto out;
    }

    openssl_ret = EVP_PKEY_get_raw_public_key(g_private_key, (unsigned char*)&g_public_key,
                                              &pubkey_size);
    if (openssl_ret <= 0) {
        eprintf("Failed to get public key: %d\n", openssl_ret);
        goto out;
    }

    ret = 0;
out:
    return ret;
}

int generate_private_key(void) {
    int ret = -1;

    if (!g_ec_ctx)
        goto out;

    eprintf("Generating enclave private key...\n");
    int openssl_ret = EVP_PKEY_keygen_init(g_ec_ctx);
    if (openssl_ret <= 0) {
        eprintf("Failed to initialize keygen: %d\n", openssl_ret);
        goto out;
    }

    openssl_ret = EVP_PKEY_keygen(g_ec_ctx, &g_private_key);
    if (openssl_ret <= 0) {
        eprintf("Failed to generate private key: %d\n", openssl_ret);
        goto out;
    }


    ret = generate_public_key();
out:
    return ret;
}

/* ECALL: initialize enclave
 * If sealed_data is provided, unseal private key from it. If not, generate new key pair.
 * Enclave public key is stored in pubkey if pubkey_size is enough for it. */
int e_initialize(uint8_t* sealed_data, size_t sealed_size, uint8_t* pubkey, size_t pubkey_size) {
    int ret = -1;

    eprintf("Enclave initializing...\n");
    OPENSSL_init_crypto(0, NULL);

    g_ec_ctx = EVP_PKEY_CTX_new_id(EC_CURVE_ID, NULL);
    if (!g_ec_ctx) {
        eprintf("Failed to create crypto context\n");
        goto out;
    }

    if (sealed_data == NULL || sealed_size == 0) {
        ret = generate_private_key();
        if (ret < 0)
            goto out;

        ret = seal_keys();
        if (ret < 0)
            goto out;
    } else {
        ret = unseal_keys(sealed_data, sealed_size);
        if (ret < 0)
            goto out;

        ret = generate_public_key();
    }

    eprintf("Enclave public key: ");
    hexdump(g_public_key);

    if (pubkey_size >= EC_PUBLIC_KEY_SIZE) {
        eprintf("Copying enclave public key...\n");
        memcpy(pubkey, &g_public_key, EC_PUBLIC_KEY_SIZE);
    }

    eprintf("Enclave initialization OK\n");
    ret = 0;

out:
    if (ret == 0)
        g_initialized = true;

    return ret;
}

// Seal enclave keys
int seal_keys(void) {
    sgx_status_t sgx_ret = SGX_ERROR_UNEXPECTED;
    uint8_t* sealed_keys = NULL;
    size_t sealed_size = 0;

    eprintf("Sealing enclave keys...\n");

    unsigned char* key_raw = NULL;
    size_t key_size = 0;
    int ret = EVP_PKEY_get_raw_private_key(g_private_key, NULL, &key_size);
    if (ret <= 0) {
        eprintf("Failed to get private key size: %d\n", ret);
        goto out;
    }

    key_raw = malloc(key_size);
    if (!key_raw) {
        eprintf("Failed to allocate memory\n");
        goto out;
    }

    ret = EVP_PKEY_get_raw_private_key(g_private_key, key_raw, &key_size);
    if (ret <= 0) {
        eprintf("Failed to get private key data: %d\n", ret);
        goto out;
    }

    // We can provide additional plaintext data to be a part of the encrypted blob's MAC if needed.
    sealed_size = sgx_calc_sealed_data_size(0, key_size);
    sealed_keys = malloc(sealed_size);
    if (!sealed_keys) {
        eprintf("Failed to allocate memory\n");
        goto out;
    }

    sgx_ret = sgx_seal_data_ex(ENCLAVE_SEALING_POLICY,
                               g_seal_attributes,
                               0, // misc mask, reserved
                               0, // additional data size
                               NULL, // no additional data
                               key_size,
                               (const uint8_t*)key_raw,
                               sealed_size,
                               (sgx_sealed_data_t*)sealed_keys);
    if (sgx_ret != SGX_SUCCESS) {
        eprintf("Failed to seal keys\n");
        goto out;
    }

    sgx_ret = o_store_sealed_data(&ret, sealed_keys, sealed_size);
    if (sgx_ret != SGX_SUCCESS || ret < 0) {
        eprintf("Failed to store sealed keys\n");
    }

out:
    // erase private key data from memory
    if (key_raw)
        zero_memory(key_raw, key_size);
    free(key_raw);

    free(sealed_keys);

    if (sgx_ret == SGX_SUCCESS && ret == 0)
        return 0;

    return -1;
}

// Restore enclave keys from sealed data
int unseal_keys(const uint8_t* sealed_data, size_t sealed_size) {
    sgx_status_t sgx_ret = SGX_ERROR_UNEXPECTED;
    uint8_t* unsealed_keys = NULL;
    uint32_t unsealed_size = 0;

    eprintf("Unsealing enclave keys...\n");

    if (sealed_size < sizeof(sgx_sealed_data_t)) {
        eprintf("Invalid sealed data\n");
        goto out;
    }

    unsealed_size = sgx_get_encrypt_txt_len((const sgx_sealed_data_t*)sealed_data);
    if (unsealed_size == UINT32_MAX) {
        eprintf("Failed to get unsealed data size\n");
        goto out;
    }

    unsealed_keys = malloc(unsealed_size);
    if (!unsealed_keys) {
        eprintf("Failed to allocate memory\n");
        goto out;
    }

    sgx_ret = sgx_unseal_data((const sgx_sealed_data_t*)sealed_data,
                              NULL, // no additional MAC data
                              0, // additional data size
                              unsealed_keys,
                              &unsealed_size);
    if (sgx_ret != SGX_SUCCESS) {
        eprintf("Failed to unseal enclave keys\n");
        goto out;
    }

    sgx_ret = SGX_ERROR_UNEXPECTED;
    // Recreate private key from the unsealed blob
    g_private_key = EVP_PKEY_new_raw_private_key(EC_CURVE_ID, NULL, unsealed_keys, unsealed_size);
    if (!g_private_key) {
        eprintf("Failed to recreate private key\n");
        goto out;
    }

    sgx_ret = SGX_SUCCESS;

out:
    if (unsealed_keys) {
        zero_memory(unsealed_keys, unsealed_size);
        free(unsealed_keys);
    }

    return sgx_ret == SGX_SUCCESS ? 0 : -1;
}

int get_report(const sgx_target_info_t* target_info, const sgx_report_data_t* report_data,
               sgx_report_t* report) {
    int ret = -1;

    if (!g_initialized)
        goto out;

    sgx_status_t sgx_ret = sgx_create_report(target_info, report_data, report);
    if (sgx_ret != SGX_SUCCESS) {
        eprintf("Failed to create enclave report: %d\n", sgx_ret);
        ret = -1;
    } else {
        ret = 0;
    }

out:
    return ret;
}

/* ECALL: get enclave report */
int e_get_report(const sgx_target_info_t* target_info, sgx_report_t* report) {
    sgx_report_data_t report_data = {0};

    // Use public key as custom data in the report
    assert(sizeof g_public_key <= sizeof report_data);
    memcpy(&report_data, g_public_key, sizeof g_public_key);

    return get_report(target_info, &report_data, report);
}

/* ECALL: sign data with enclave's private key */
int e_sign_data(const void* data, size_t data_size, void* signature, size_t signature_size) {
    int ret = -1;
    EVP_MD_CTX* md = NULL;

    if (!g_initialized)
        goto out;

    if (signature_size != EC_SIGNATURE_SIZE) {
        eprintf("Invalid signature size %zu, expected %zu\n", signature_size, EC_SIGNATURE_SIZE);
        goto out;
    }

    md = EVP_MD_CTX_create();
    if (!md)
        goto out;

    ret = EVP_DigestSignInit(md, NULL, NULL, NULL, g_private_key);
    if (ret != 1) {
        ret = -1;
        goto out;
    }

    ret = EVP_DigestSign(md, signature, &signature_size, data, data_size);
    if (ret != 1) {
        ret = -1;
        goto out;
    }

    eprintf("Signed %zu bytes of data\n", data_size);
    ret = 0;
out:
    if (md)
        EVP_MD_CTX_destroy(md);
    return ret;
}
