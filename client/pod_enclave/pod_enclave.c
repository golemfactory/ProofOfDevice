#include <assert.h>
#include <openssl/pem.h>
#include <sgx_utils.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>
#include <tSgxSSL_api.h>

#include "pod_enclave.h"
#include "pod_enclave_t.h"
#include "pod_enclave_mrsigner.h"

bool g_initialized = false;
EVP_PKEY* g_private_key = NULL;
RSA* g_private_key_rsa = NULL;
char* g_private_key_pem = NULL;
uint8_t g_pub_key_hash[SHA256_DIGEST_LENGTH];

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

// Generate PEM from PKEY, caller must free pem
int generate_pem(EVP_PKEY* key, bool private, char** pem) {
    BIO* bio = NULL;
    int pem_size = 0;
    int ret = -1;

    if (!key) {
        eprintf("Key not initialized\n");
        goto out;
    }

    if (*pem) {
        eprintf("PEM not NULL\n");
        goto out;
    }

    // read key as PEM
    bio = BIO_new(BIO_s_mem());
    if (!bio) {
        eprintf("Failed to allocate memory\n");
        goto out;
    }

    if (private) {
        if (PEM_write_bio_PrivateKey(bio, key, NULL, NULL, 0, NULL, NULL) != 1) {
            eprintf("Failed to generate private key PEM\n");
            goto out;
        }
    } else {
        if (PEM_write_bio_PUBKEY(bio, key) != 1) {
            eprintf("Failed to generate public key PEM\n");
            goto out;
        }
    }

    pem_size = BIO_pending(bio);
    *pem = malloc(pem_size + 1); // 1 for NULL terminator
    if (!*pem) {
        eprintf("Failed to allocate memory\n");
        goto out;
    }

    zero_memory(*pem, pem_size + 1);

    ret = BIO_read(bio, *pem, pem_size);
    if (ret != pem_size) {
        ret = -1;
        eprintf("Failed to read key PEM\n");
        goto out;
    }

    ret = 0;

out:
    if (bio)
        BIO_free_all(bio);

    if (ret < 0) {
        if (*pem) {
            zero_memory(*pem, pem_size + 1);
            free(*pem);
            *pem = NULL;
        }
    }

    return ret;
}

// Generate RSA key pair
int generate_private_key(void) {
    int ret = -1;
    BIGNUM* exponent = NULL;

    eprintf("Generating enclave key...\n");
    exponent = BN_new();
    if (!exponent) {
        eprintf("Failed to allocate memory\n");
        goto out;
    }

    // use 65537 exponent
    if (BN_set_word(exponent, RSA_F4) != 1) {
        eprintf("Failed to set RSA key exponent\n");
        goto out;
    }

    g_private_key_rsa = RSA_new();
    if (!g_private_key_rsa) {
        eprintf("Failed to allocate memory\n");
        goto out;
    }

    // 4096 bits
    if (RSA_generate_key_ex(g_private_key_rsa, 4096, exponent, NULL) != 1) {
        eprintf("Failed to generate RSA key\n");
        goto out;
    }

    if (EVP_PKEY_assign_RSA(g_private_key, g_private_key_rsa) != 1) {
        eprintf("Failed to assign RSA key\n");
        goto out;
    }

    // generate PEM for the key
    if (generate_pem(g_private_key, true, &g_private_key_pem) < 0) {
        eprintf("Failed to generate PEM key\n");
        goto out;
    }

    ret = 0;

out:
    if (exponent)
        BN_free(exponent);

    if (ret < 0) {
        // erase keys, reset to uninitialized
        if (g_private_key)
            EVP_PKEY_free(g_private_key);
        if (g_private_key_rsa)
            RSA_free(g_private_key_rsa);
        g_private_key = NULL;
        g_private_key_rsa = NULL;

        if (g_private_key_pem) {
            zero_memory(g_private_key_pem, strlen(g_private_key_pem) + 1);
            free(g_private_key_pem);
            g_private_key_pem = NULL;
        }

        g_initialized = false;
    }

    return ret;
}

/* ECALL: initialize enclave
 * If sealed_data is provided, unseal private key from it. If not, generate new key pair.
 * If export_pubkey is true, call o_store_public_key() OCALL. */
int e_initialize(uint8_t* sealed_data, size_t sealed_size, bool export_pubkey) {
    int ret = -1;
    sgx_status_t sgx_ret = SGX_ERROR_UNEXPECTED;
    uint8_t* sealed_keys = NULL;
    char* pub_pem = NULL;
    SHA256_CTX sha;

    eprintf("Enclave initializing...\n");
    OPENSSL_init_crypto(0, NULL);

    g_private_key = EVP_PKEY_new();
    if (!g_private_key) {
        eprintf("Failed to allocate memory\n");
        goto out;
    }

    if (sealed_data == NULL || sealed_size == 0) {
        ret = generate_private_key();
        if (ret < 0) {
            eprintf("Failed to generate enclave key\n");
            goto out;
        }

        ret = seal_keys(&sealed_data, &sealed_size);
        if (ret < 0) {
            eprintf("Failed to seal keys\n");
            goto out;
        }
        sealed_keys = sealed_data; // save for freeing

        sgx_ret = o_store_sealed_data(&ret, sealed_data, sealed_size);
        if (sgx_ret != SGX_SUCCESS || ret < 0) {
            ret = -1;
            eprintf("Failed to store sealed keys\n");
            goto out;
        }
    }
    else {
        ret = unseal_keys(sealed_data);
        if (ret < 0) {
            eprintf("Failed to unseal keys\n");
            goto out;
        }
    }

    // generate PEM-formatted public key
    ret = generate_pem(g_private_key, false, &pub_pem);
    if (ret < 0) {
        eprintf("Failed to generate public key PEM\n");
        goto out;
    }

    if (export_pubkey) {
        sgx_ret = o_store_public_key(&ret, (uint8_t*)pub_pem, strlen(pub_pem));
        if (sgx_ret != SGX_SUCCESS || ret < 0) {
            ret = -1;
            eprintf("Failed to store public key\n");
            goto out;
        }
    }

    // calculate pub key hash
    ret = -1;
    if (SHA256_Init(&sha) != 1) {
        eprintf("Failed to init digest context\n");
        goto out;
    }

    if (SHA256_Update(&sha, pub_pem, strlen(pub_pem)) != 1) {
        eprintf("Failed to calculate public key hash\n");
        goto out;
    }

    if (SHA256_Final(g_pub_key_hash, &sha) != 1) {
        eprintf("Failed to finalize public key hash\n");
        goto out;
    }

    eprintf("Public enclave key hash: ");
    hexdump(g_pub_key_hash);

    eprintf("Enclave initialization OK\n");

    ret = 0;

out:
    free(sealed_keys);
    free(pub_pem);

    if (ret == 0)
        g_initialized = true;

    return ret;
}

// Seal enclave keys
int seal_keys(uint8_t** sealed_keys, size_t* sealed_size) {
    sgx_status_t sgx_ret = SGX_ERROR_UNEXPECTED;
    size_t unsealed_size = 0;

    eprintf("Sealing keys...\n");
    unsealed_size = strlen(g_private_key_pem) + 1; // 1 for NULL terminator

    // We can provide additional plaintext data to be a part of the encrypted blob's MAC if needed.
    *sealed_size = sgx_calc_sealed_data_size(0, unsealed_size);
    *sealed_keys = malloc(*sealed_size);
    if (!*sealed_keys) {
        eprintf("Failed to allocate memory\n");
        goto out;
    }

    sgx_ret = sgx_seal_data_ex(ENCLAVE_SEALING_POLICY,
                               g_seal_attributes,
                               0, // misc mask, reserved
                               0, // additional data size
                               NULL, // no additional data
                               unsealed_size,
                               (const uint8_t*)g_private_key_pem,
                               *sealed_size,
                               (sgx_sealed_data_t*)*sealed_keys);

out:
    if (sgx_ret != SGX_SUCCESS) {
        zero_memory(*sealed_keys, *sealed_size);
        free(*sealed_keys);
        *sealed_size = 0;
        return -1;
    }

    return 0;
}

// Restore enclave keys from sealed data
int unseal_keys(const uint8_t* sealed_data) {
    sgx_status_t sgx_ret = SGX_ERROR_UNEXPECTED;
    uint8_t* unsealed_keys = NULL;
    uint32_t unsealed_size = 0;
    BIO* bio = NULL;

    eprintf("Unsealing enclave keys...\n");
    unsealed_size = sgx_get_encrypt_txt_len((const sgx_sealed_data_t*)sealed_data);
    if (unsealed_size == 0xffffffff) {
        eprintf("Failed to get sealed data size\n");
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
        eprintf("Failed to unseal enclave key\n");
        goto out;
    }

    sgx_ret = SGX_ERROR_UNEXPECTED;
    // Load private key from unsealed PEM blob
    bio = BIO_new_mem_buf(unsealed_keys, -1);
    if (bio == NULL) {
        eprintf("Failed to load enclave key\n");
        goto out;
    }

    g_private_key = PEM_read_bio_PrivateKey(bio, NULL, NULL, NULL);
    if (g_private_key == NULL) {
        eprintf("Failed to parse enclave key\n");
        goto out;
    }

    g_private_key_rsa = EVP_PKEY_get1_RSA(g_private_key);
    if (g_private_key_rsa == NULL) {
        eprintf("Failed to parse enclave key\n");
        goto out;
    }

    sgx_ret = SGX_SUCCESS;

out:
    if (bio)
        BIO_free_all(bio);

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

    // Use pub key hash as custom data in the report
    assert(sizeof g_pub_key_hash <= sizeof report_data);
    memcpy(&report_data, g_pub_key_hash, sizeof g_pub_key_hash);

    return get_report(target_info, &report_data, report);
}

/* ECALL: sign data with enclave's RSA key */
int e_sign_data(const void* data, size_t data_size, void* signature, size_t signature_size) {
    int ret = -1;
    EVP_MD_CTX* md = NULL;

    if (!g_initialized)
        goto out;

    md = EVP_MD_CTX_create();
    if (!md)
        goto out;

    ret = EVP_DigestSignInit(md, NULL, EVP_sha256(), NULL, g_private_key);
    if (ret != 1) {
        ret = -1;
        goto out;
    }

    ret = EVP_DigestSignUpdate(md, data, data_size);
    if (ret != 1) {
        ret = -1;
        goto out;
    }

    // get signature size
    size_t needed_size;
    ret = EVP_DigestSignFinal(md, NULL, &needed_size);
    if (ret != 1) {
        ret = -1;
        goto out;
    }

    ret = -1;
    if (needed_size != signature_size) {
        eprintf("Invalid signature size %zu, expected %zu\n", signature_size, needed_size);
        goto out;
    }

    ret = EVP_DigestSignFinal(md, signature, &needed_size);
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

/* ECALL: get size needed for digital signature */
int e_get_signature_size(const void* data, size_t data_size, size_t* signature_size) {
    int ret = -1;
    EVP_MD_CTX* md = NULL;

    if (!g_initialized)
        goto out;

    md = EVP_MD_CTX_create();
    if (!md)
        goto out;

    ret = EVP_DigestSignInit(md, NULL, EVP_sha256(), NULL, g_private_key);
    if (ret != 1) {
        ret = -1;
        goto out;
    }

    ret = EVP_DigestSignUpdate(md, data, data_size);
    if (ret != 1) {
        ret = -1;
        goto out;
    }

    ret = EVP_DigestSignFinal(md, NULL, signature_size);
    if (ret != 1) {
        ret = -1;
        goto out;
    }

    eprintf("Signature size: %zu bytes\n", *signature_size);
    ret = 0;
out:
    if (md)
        EVP_MD_CTX_destroy(md);
    return ret;
}
