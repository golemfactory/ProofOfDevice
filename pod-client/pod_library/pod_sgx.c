#include <ctype.h>
#include <errno.h>
#include <openssl/sha.h>
#include <stdbool.h>
#include <stdio.h>
#include <sys/stat.h>

#include <sgx_uae_service.h>

#include "pod_sgx.h"
#include "pod_enclave.h"
#include "pod_enclave_u.h"

ssize_t get_file_size(int fd) {
    struct stat st;

    if (fstat(fd, &st) != 0)
        return -1;

    return st.st_size;
}

void* read_file(void* buffer, const char* path, size_t* size) {
    FILE* f = NULL;
    ssize_t fs = 0;
    void* buf = buffer;

    if (!size || !path)
        return NULL;

    f = fopen(path, "rb");
    if (!f) {
        fprintf(stderr, "Failed to open file '%s' for reading: %s\n", path, strerror(errno));
        goto out;
    }

    if (*size == 0) { // read whole file
        fs = get_file_size(fileno(f));
        if (fs < 0) {
            fprintf(stderr, "Failed to get size of file '%s': %s\n", path, strerror(errno));
            goto out;
        }
    } else {
        fs = *size;
    }

    if (!buffer) {
        buffer = malloc(fs);
        if (!buffer) {
            fprintf(stderr, "No memory\n");
            goto out;
        }
    }

    if (fread(buffer, fs, 1, f) != 1) {
        fprintf(stderr, "Failed to read file '%s'\n", path);
        if (!buf) {
            free(buffer);
            buffer = NULL;
        }
    }

out:
    if (f)
        fclose(f);

    if (*size == 0)
        *size = fs;

    return buffer;
}

int write_file(const char* path, size_t size, const void* buffer) {
    FILE* f = NULL;
    int status;

    f = fopen(path, "wb");
    if (!f) {
        fprintf(stderr, "Failed to open file '%s' for writing: %s\n", path, strerror(errno));
        goto out;
    }

    if (size > 0 && buffer) {
        if (fwrite(buffer, size, 1, f) != 1) {
            fprintf(stderr, "Failed to write file '%s': %s\n", path, strerror(errno));
            goto out;
        }
    }

    errno = 0;

out:
    status = errno;
    if (f)
        fclose(f);
    return status;
}

/* Returns 0 on failure */
static sgx_enclave_id_t enclave_load(const char* enclave_path, bool debug_enabled) {
    int is_token_updated = 0;
    sgx_launch_token_t launch_token = {0};
    sgx_misc_attribute_t misc_attribs = {0};
    sgx_enclave_id_t enclave_id = 0;

    printf("Loading enclave from file '%s'\n", enclave_path);

    sgx_status_t sgx_ret = sgx_create_enclave(enclave_path, debug_enabled, &launch_token,
                                              &is_token_updated, &enclave_id, &misc_attribs);
    if (sgx_ret != SGX_SUCCESS) {
        fprintf(stderr, "Failed to load enclave: %d\n", sgx_ret);
    } else {
        printf("Enclave loaded successfully, id = 0x%lx\n", enclave_id);
    }

    return enclave_id;
}

static sgx_status_t enclave_unload(sgx_enclave_id_t enclave_id) {
    sgx_status_t sgx_ret = sgx_destroy_enclave(enclave_id);
    if (sgx_ret != SGX_SUCCESS)
        fprintf(stderr, "Failed to unload enclave\n");
    else
        printf("Enclave unloaded\n");

    return sgx_ret;
}

static sgx_enclave_id_t g_enclave_id = 0;
static uint8_t* g_sealed_state = NULL;
static size_t g_sealed_state_size = 0;

static int load_pod_enclave_fresh(const char* enclave_path, bool debug_enabled,
                                  uint8_t* sealed_state, size_t sealed_state_size) {
    int ret = -1;
    uint8_t* sealed_keys = NULL;

    if (g_enclave_id != 0) {
        fprintf(stderr, "Enclave already loaded with id %lu\n", g_enclave_id);
        goto out;
    }

    g_sealed_state = sealed_state;
    g_sealed_state_size = sealed_state_size;

    g_enclave_id = enclave_load(enclave_path, debug_enabled);
    if (g_enclave_id == 0)
        goto out;

    size_t sealed_size = 0;

    // ECALL: enclave initialization
    sgx_status_t sgx_ret = e_initialize(g_enclave_id, &ret, sealed_keys, sealed_size, NULL, 0);

    if (sgx_ret != SGX_SUCCESS) {
        fprintf(stderr, "Failed to call enclave initialization\n");
        goto out;
    }

    if (ret < 0) {
        fprintf(stderr, "Enclave initialization failed\n");
        goto out;
    }

    ret = g_sealed_state_size;
out:
    free(sealed_keys);
    return ret;
}

static int load_pod_enclave_from_state(const char* enclave_path, bool debug_enabled,
                                       const uint8_t* sealed_state, size_t sealed_state_size) {
    int ret = -1;

    if (g_enclave_id != 0) {
        fprintf(stderr, "Enclave already loaded with id %lu\n", g_enclave_id);
        goto out;
    }

    g_enclave_id = enclave_load(enclave_path, debug_enabled);
    if (g_enclave_id == 0)
        goto out;

    printf("Loading sealed enclave state from provided buffer\n");

    // ECALL: enclave initialization
    sgx_status_t sgx_ret = e_initialize(g_enclave_id, &ret, (uint8_t*) sealed_state, sealed_state_size, NULL, 0);

    if (sgx_ret != SGX_SUCCESS) {
        fprintf(stderr, "Failed to call enclave initialization\n");
        goto out;
    }

    if (ret < 0) {
        fprintf(stderr, "Enclave initialization failed\n");
        goto out;
    }

    ret = 0;
out:
    return ret;
}

static int generate_enclave_quote(sgx_spid_t sp_id, sgx_quote_sign_type_t quote_type,
                                  uint8_t* quote_buffer, size_t quote_buffer_size) {
    int ret = -1;
    sgx_status_t sgx_ret = SGX_ERROR_UNEXPECTED;
    sgx_epid_group_id_t epid_group_id = { 0 };
    sgx_target_info_t qe_info = { 0 };
    sgx_report_t report = { 0 };
    sgx_quote_nonce_t qe_nonce = { 0 };
    sgx_report_t qe_report = { 0 };
    uint32_t quote_size = 0;

    if (g_enclave_id == 0) {
        fprintf(stderr, "Enclave not loaded\n");
        goto out;
    }

    // Initialize the quoting process, get quoting enclave info
    sgx_ret = sgx_init_quote(&qe_info, &epid_group_id);
    if (sgx_ret != SGX_SUCCESS) {
        fprintf(stderr, "Failed to initialize quoting process\n");
        goto out;
    }

    // TODO: use revocation list from IAS if available
    sgx_ret = sgx_calc_quote_size(NULL, 0, &quote_size);

    if (sgx_ret != SGX_SUCCESS) {
        fprintf(stderr, "Failed to calculate quote size\n");
        goto out;
    }

    if (quote_buffer_size < quote_size) {
        fprintf(stderr, "Provided buffer size is too small to fit the quote of size %d\n", quote_size);
        ret = -1;
        goto out;
    }

    // ECALL: generate enclave's report, targeted to Quoting Enclave (QE)
    sgx_ret = e_get_report(g_enclave_id, &ret, &qe_info, &report);
    if (sgx_ret != SGX_SUCCESS || ret < 0) {
        ret = -1;
        fprintf(stderr, "Failed to get enclave's report\n");
        goto out;
    }

    // Prepare random nonce
    // TODO: ideally this nonce would be received from a 3rd party on a different system
    // that will verify the QE report
    size_t nonce_size = sizeof(qe_nonce);
    if (!read_file(&qe_nonce, "/dev/urandom", &nonce_size)) {
        ret = -1;
        goto out;
    }

    // Get enclave's quote. TODO: use revocation list
    sgx_ret = sgx_get_quote(&report,
                            quote_type,
                            &sp_id, // service provider id
                            &qe_nonce, // nonce for QE report
                            NULL, // no revocation list
                            0, // revocation list size
                            &qe_report, // optional QE report
                            (sgx_quote_t*) quote_buffer,
                            quote_size);

    if (sgx_ret != SGX_SUCCESS) {
        fprintf(stderr, "Failed to get enclave quote: %d\n", sgx_ret);
        goto out;
    }

    // Calculate expected qe_report.body.report_data
    // It should be sha256(nonce||quote)
    ret = -1;
    uint8_t hash[SHA256_DIGEST_LENGTH];
    SHA256_CTX sha;

    if (SHA256_Init(&sha) != 1) {
        fprintf(stderr, "Failed to init digest context\n");
        goto out;
    }

    if (SHA256_Update(&sha, &qe_nonce, sizeof(qe_nonce)) != 1) {
        fprintf(stderr, "Failed to calculate hash\n");
        goto out;
    }

    if (SHA256_Update(&sha, (sgx_quote_t*) quote_buffer, quote_size) != 1) {
        fprintf(stderr, "Failed to calculate hash\n");
        goto out;
    }

    if (SHA256_Final(hash, &sha) != 1) {
        fprintf(stderr, "Failed to finalize hash\n");
        goto out;
    }

    if (memcmp(&qe_report.body.report_data, hash, sizeof(hash)) != 0) {
        fprintf(stderr, "Quoting Enclave report contains invalid data\n");
        goto out;
    }

    ret = quote_size;
out:
    return ret;
}

int pod_init_enclave(const char* enclave_path, uint8_t* sealed_state, size_t sealed_state_size) {
    return load_pod_enclave_fresh(enclave_path, ENCLAVE_DEBUG_ENABLED, sealed_state, sealed_state_size);
}

int pod_load_enclave(const char* enclave_path, const uint8_t* sealed_state, size_t sealed_state_size) {
    return load_pod_enclave_from_state(enclave_path, ENCLAVE_DEBUG_ENABLED, sealed_state, sealed_state_size);
}

int pod_unload_enclave(void) {
    if (g_enclave_id == 0)
        return 0;
    int ret = enclave_unload(g_enclave_id);
    if (ret == 0)
        g_enclave_id = 0;
    return ret;
}

int pod_get_quote(const char* sp_id_str, const char* sp_quote_type_str, uint8_t* quote_buffer,
                  size_t quote_buffer_size) {
    sgx_spid_t sp_id = { 0 };
    sgx_quote_sign_type_t sp_quote_type;
    int ret = -1;

    // parse SPID
    if (strlen(sp_id_str) != 32) {
        fprintf(stderr, "Invalid SPID: %s\n", sp_id_str);
        goto out;
    }

    for (int i = 0; i < 16; i++) {
        if (!isxdigit(sp_id_str[i * 2]) || !isxdigit(sp_id_str[i * 2 + 1])) {
            fprintf(stderr, "Invalid SPID: %s\n", sp_id_str);
            goto out;
        }

        sscanf(sp_id_str + i * 2, "%02hhx", &sp_id.id[i]);
    }

    // parse quote type
    if (*sp_quote_type_str == 'l' || *sp_quote_type_str == 'L') {
        sp_quote_type = SGX_LINKABLE_SIGNATURE;
    } else if (*sp_quote_type_str == 'u' || *sp_quote_type_str == 'U') {
        sp_quote_type = SGX_UNLINKABLE_SIGNATURE;
    } else {
        fprintf(stderr, "Invalid quote type: %s\n", sp_quote_type_str);
        goto out;
    }

    ret = generate_enclave_quote(sp_id, sp_quote_type, quote_buffer, quote_buffer_size);
out:
    return ret;
}

int pod_sign_buffer(const void* data, size_t data_size, void* signature, size_t signature_size) {
    int ret = -1;

    if (g_enclave_id == 0) {
        fprintf(stderr, "PoD enclave not loaded\n");
        goto out;
    }

    if (!data || data_size == 0) {
        fprintf(stderr, "Invalid data buffer\n");
        goto out;
    }

    if (!signature || signature_size == 0) {
        fprintf(stderr, "Invalid signature buffer\n");
        goto out;
    }

    // ECALL: sign data
    sgx_status_t sgx_ret = e_sign_data(g_enclave_id, &ret, data, data_size, signature,
                                       signature_size);
    if (sgx_ret != SGX_SUCCESS || ret < 0) {
        ret = -1;
        fprintf(stderr, "Failed to sign data\n");
        goto out;
    }

out:
    return ret;
}

int pod_sign_file(const char* input_path, const char* signature_path) {
    int ret = -1;
    uint8_t signature[EC_SIGNATURE_SIZE];

    if (!input_path || !signature_path) {
        fprintf(stderr, "Invalid path\n");
        goto out;
    }

    size_t input_size = 0;
    void* input = read_file(NULL, input_path, &input_size);
    if (!input)
        goto out;

    ret = pod_sign_buffer(input, input_size, &signature, sizeof(signature));
    if (ret < 0)
        goto out;

    ret = write_file(signature_path, sizeof(signature), &signature);
    if (ret < 0)
        goto out;

    printf("Saved signature to '%s'\n", signature_path);
    ret = 0;

out:
    return ret;
}

// OCALL: save sealed enclave state
int o_store_sealed_data(const uint8_t* sealed_data, size_t sealed_size) {
    printf("Saving sealed enclave state to provided buffer\n");

    if (g_sealed_state_size < sealed_size) {
      printf("Provided buffer is too small to fit required size: %ld\n", sealed_size);
      return -1;
    }

    memcpy(g_sealed_state, sealed_data, sealed_size);
    g_sealed_state_size = sealed_size;
    return 0;
}

// OCALL: print string
void o_print(const char* str) {
    printf("%s", str);
}
