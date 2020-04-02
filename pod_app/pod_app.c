#include <ctype.h>
#include <errno.h>
#include <getopt.h>
#include <openssl/sha.h>
#include <stdbool.h>
#include <stdio.h>
#include <sys/stat.h>

#include <sgx_uae_service.h>

#include "pod_app.h"
#include "pod_enclave_u.h"

struct option g_options[] = {
    { "help", no_argument, 0, 'h' },
    { "sealed-path", required_argument, 0, 's' },
    { "enclave-path", required_argument, 0, 'e' },
    { "pubkey-path", required_argument, 0, 'p' },
    { "spid", required_argument, 0, 'i' },
    { "quote-type", required_argument, 0, 't' },
    { "quote-path", required_argument, 0, 'q' },
    { "data-path", required_argument, 0, 'D' },
    { "sig-path", required_argument, 0, 'S' },
    { 0, 0, 0, 0 }
};

void usage(const char* exec) {
    printf("%s mode [options]\n", exec);
    printf("Available modes:\n");
    printf("  init                     Generate a private enclave key pair and export its public part,\n");
    printf("                           generate enclave quote and export it\n");
    printf("  sign                     Sign provided data with enclave's private key\n");
    printf("Available general options:\n");
    printf("  --help, -h               Display this help\n");
    printf("  --sealed-path, -s PATH   Path for sealed keys storage, default: " DEFAULT_SEALED_KEYS_PATH "\n");
    printf("  --enclave-path, -e PATH  Path for enclave binary, default: " DEFAULT_ENCLAVE_PATH "\n");
    printf("Available init options:\n");
    printf("  --pubkey-path, -p PATH   Path to save enclave public key to, default: " DEFAULT_PUBLIC_KEY_PATH "\n");
    printf("  --spid, -i SPID          Service Provider ID received during IAS registration (hex string)\n");
    printf("  --quote-type, -t TYPE    Service Provider quote type, (l)inkable or (u)nlinkable)\n");
    printf("  --quote-path, -q PATH    Path to save enclave quote to, default: " DEFAULT_ENCLAVE_QUOTE_PATH "\n");
    printf("Available sign options:\n");
    printf("  --data, -D PATH          Path to file with data to sign\n");
    printf("  --sig-path, -S PATH      Path to save generated signature to\n");
}

/* Return -1 on error */
ssize_t get_file_size(int fd) {
    struct stat st;

    if (fstat(fd, &st) != 0)
        return -1;

    return st.st_size;
}

/*!
 *  \brief Read file contents
 *
 *  \param[in]     buffer Buffer to read data to. If NULL, this function allocates one.
 *  \param[in]     path   Path to the file.
 *  \param[in,out] size   On entry, number of bytes to read. 0 means to read the entire file.
 *                        On exit, number of bytes read.
 *  \return On success, pointer to the data buffer. If \p buffer was NULL, caller should free this.
 *          On failure, NULL.
 */
void* read_file(void* buffer, const char* path, size_t* size) {
    FILE* f = NULL;
    ssize_t fs = 0;
    void* buf = buffer;

    if (!size || !path)
        return NULL;

    f = fopen(path, "rb");
    if (!f) {
        printf("Failed to open file '%s' for reading: %s\n", path, strerror(errno));
        goto out;
    }

    if (*size == 0) { // read whole file
        fs = get_file_size(fileno(f));
        if (fs < 0) {
            printf("Failed to get size of file '%s': %s\n", path, strerror(errno));
            goto out;
        }
    } else {
        fs = *size;
    }

    if (!buffer) {
        buffer = malloc(fs);
        if (!buffer) {
            printf("No memory\n");
            goto out;
        }
    }

    if (fread(buffer, fs, 1, f) != 1) {
        printf("Failed to read file '%s'\n", path);
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

/* Write buffer to file */
int write_file(const char* path, size_t size, const void* buffer) {
    FILE* f = NULL;
    int status;

    f = fopen(path, "wb");
    if (!f) {
        printf("Failed to open file '%s' for writing: %s\n", path, strerror(errno));
        goto out;
    }

    if (size > 0 && buffer) {
        if (fwrite(buffer, size, 1, f) != 1) {
            printf("Failed to write file '%s': %s\n", path, strerror(errno));
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
sgx_enclave_id_t enclave_load(const char* enclave_path, bool debug_enabled) {
    int is_token_updated = 0;
    sgx_launch_token_t launch_token = {0};
    sgx_misc_attribute_t misc_attribs = {0};
    sgx_enclave_id_t enclave_id = 0;

    printf("Loading enclave from file '%s'\n", enclave_path);

    sgx_status_t sgx_ret = sgx_create_enclave(enclave_path, debug_enabled, &launch_token,
                                              &is_token_updated, &enclave_id, &misc_attribs);
    if (sgx_ret != SGX_SUCCESS) {
        printf("Failed to load enclave: %d\n", sgx_ret);
    } else {
        printf("Enclave loaded successfully, id = 0x%lx\n", enclave_id);
    }

    return enclave_id;
}

sgx_status_t enclave_unload(sgx_enclave_id_t enclave_id) {
    sgx_status_t sgx_ret = sgx_destroy_enclave(enclave_id);
    if (sgx_ret != SGX_SUCCESS)
        printf("Failed to unload enclave\n");
    else
        printf("Enclave unloaded\n");

    return sgx_ret;
}

static sgx_enclave_id_t g_enclave_id = 0;
static const char* g_sealed_state_path = NULL;
static const char* g_public_key_path = NULL;

int load_pod_enclave(const char* enclave_path, bool debug_enabled, const char* sealed_state_path,
                     bool load_sealed_state, const char* public_key_path) {
    int ret = -1;
    uint8_t* sealed_keys = NULL;

    if (g_enclave_id != 0) {
        printf("Enclave already loaded with id %lu\n", g_enclave_id);
        goto out;
    }

    g_sealed_state_path = sealed_state_path;
    g_public_key_path = public_key_path;

    g_enclave_id = enclave_load(enclave_path, debug_enabled);
    if (g_enclave_id == 0)
        goto out;

    size_t sealed_size = 0;

    if (load_sealed_state) {
        printf("Loading sealed enclave state from '%s'\n", sealed_state_path);
        sealed_keys = read_file(NULL, sealed_state_path, &sealed_size); // may return NULL
        if (sealed_keys == NULL)
            goto out;
    }

    // ECALL: enclave initialization
    sgx_status_t sgx_ret = e_initialize(g_enclave_id, &ret, sealed_keys, sealed_size,
                                        public_key_path != NULL);
    if (sgx_ret != SGX_SUCCESS) {
        printf("Failed to call enclave initialization\n");
        goto out;
    }

    if (ret < 0) {
        printf("Enclave initialization failed\n");
        goto out;
    }

    ret = 0;
out:
    free(sealed_keys);
    return ret;
}

int generate_enclave_quote(sgx_spid_t sp_id, sgx_quote_sign_type_t quote_type,
                           const char* quote_path) {
    int ret = -1;
    sgx_status_t sgx_ret = SGX_ERROR_UNEXPECTED;
    sgx_epid_group_id_t epid_group_id = { 0 };
    sgx_target_info_t qe_info = { 0 };
    sgx_report_t report = { 0 };
    sgx_quote_nonce_t qe_nonce = { 0 };
    sgx_report_t qe_report = { 0 };
    uint32_t quote_size = 0;
    sgx_quote_t* quote = NULL;

    if (g_enclave_id == 0) {
        printf("Enclave not loaded\n");
        goto out;
    }

    // Initialize the quoting process, get quoting enclave info
    sgx_ret = sgx_init_quote(&qe_info, &epid_group_id);
    if (sgx_ret != SGX_SUCCESS) {
        printf("Failed to initialize quoting process\n");
        goto out;
    }

    // TODO: use revocation list from IAS if available
    sgx_ret = sgx_calc_quote_size(NULL, 0, &quote_size);

    if (sgx_ret != SGX_SUCCESS) {
        printf("Failed to calculate quote size\n");
        goto out;
    }

    quote = malloc(quote_size);
    if (!quote) {
        printf("Failed to allocate memory for quote\n");
        goto out;
    }

    // ECALL: generate enclave's report, targeted to Quoting Enclave (QE)
    sgx_ret = e_get_report(g_enclave_id, &ret, &qe_info, &report);
    if (sgx_ret != SGX_SUCCESS || ret < 0) {
        ret = -1;
        printf("Failed to get enclave's report\n");
        goto out;
    }

    // Prepare random nonce
    // TODO: ideally this nonce would be received from a 3rd party on a different system
    // that will verify the QE report
    size_t nonce_size = sizeof(qe_nonce);
    if (!read_file(&qe_nonce, "/dev/urandom", &nonce_size)) {
        ret = -1;
        printf("Failed to read random data\n");
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
                            quote,
                            quote_size);

    if (sgx_ret != SGX_SUCCESS) {
        printf("Failed to get enclave quote: %d\n", sgx_ret);
        goto out;
    }

    // Calculate expected qe_report.body.report_data
    // It should be sha256(nonce||quote)
    ret = -1;
    uint8_t hash[SHA256_DIGEST_LENGTH];
    SHA256_CTX sha;

    if (SHA256_Init(&sha) != 1) {
        printf("Failed to init digest context\n");
        goto out;
    }

    if (SHA256_Update(&sha, &qe_nonce, sizeof(qe_nonce)) != 1) {
        printf("Failed to calculate hash\n");
        goto out;
    }

    if (SHA256_Update(&sha, quote, quote_size) != 1) {
        printf("Failed to calculate hash\n");
        goto out;
    }

    if (SHA256_Final(hash, &sha) != 1) {
        printf("Failed to finalize hash\n");
        goto out;
    }

    if (memcmp(&qe_report.body.report_data, hash, sizeof(hash)) != 0) {
        printf("Quoting Enclave report contains invalid data\n");
        goto out;
    }

    if (write_file(quote_path, quote_size, quote) == 0) {
        printf("Enclave quote saved to '%s'\n", quote_path);
    } else {
        goto out;
    }

    ret = 0;
out:
    free(quote);
    return ret;
}

int main(int argc, char* argv[]) {
    int this_option = 0;
    bool sp_id_set = false;
    sgx_spid_t sp_id = {{ 0 }};
    sgx_quote_sign_type_t sp_quote_type;
    bool quote_type_set = false;
    char* sealed_keys_path = DEFAULT_SEALED_KEYS_PATH;
    char* public_key_path = DEFAULT_PUBLIC_KEY_PATH;
    char* enclave_path = DEFAULT_ENCLAVE_PATH;
    char* quote_path = DEFAULT_ENCLAVE_QUOTE_PATH;
    char* data_path = NULL;
    char* sig_path = NULL;
    char* mode = NULL;
    int ret = -1;

    while (true) {
        this_option = getopt_long(argc, argv, "hs:e:p:i:t:q:D:S:", g_options, NULL);

        if (this_option == -1)
            break;

        switch (this_option) {
            case 'h':
                usage(argv[0]);
                exit(0);
            case 's':
                sealed_keys_path = optarg;
                break;
            case 'e':
                enclave_path = optarg;
                break;
            case 'p':
                public_key_path = optarg;
                break;
            case 'i': {
                if (strlen(optarg) != 32) {
                    printf("Invalid SPID: %s\n", optarg);
                    goto out;
                }

                for (int i = 0; i < 16; i++) {
                    if (!isxdigit(optarg[i * 2]) || !isxdigit(optarg[i * 2 + 1])) {
                        printf("Invalid SPID: %s\n", optarg);
                        goto out;
                    }

                    sscanf(optarg + i * 2, "%02hhx", &sp_id.id[i]);
                }
                sp_id_set = true;
                break;
            }
            case 't':
                if (*optarg == 'l' || *optarg == 'L') {
                    sp_quote_type = SGX_LINKABLE_SIGNATURE;
                    quote_type_set = true;
                } else if (*optarg == 'u' || *optarg == 'U') {
                    sp_quote_type = SGX_UNLINKABLE_SIGNATURE;
                    quote_type_set = true;
                } else {
                    printf("Invalid quote type: %s\n", optarg);
                    goto out;
                }
                break;
            case 'q':
                quote_path = optarg;
                break;
            case 'D':
                data_path = optarg;
                break;
            case 'S':
                sig_path = optarg;
                break;
            default:
                printf("Unknown option: %c\n", this_option);
                usage(argv[0]);
                goto out;
        }
    }

    if (optind >= argc) {
        printf("Mode not specified\n");
        usage(argv[0]);
        goto out;
    }

    mode = argv[optind++];

    switch (mode[0]) {
        case 'i': { // init
            if (!sp_id_set) {
                printf("SPID not set\n");
                usage(argv[0]);
                goto out;
            }

            if (!quote_type_set) {
                printf("Quote type not set\n");
                usage(argv[0]);
                goto out;
            }

            ret = load_pod_enclave(enclave_path,
                                   ENCLAVE_DEBUG_ENABLED,
                                   sealed_keys_path,
                                   false, // don't load existing sealed state
                                   public_key_path); // export public key
            if (ret < 0)
                goto out;

            ret = generate_enclave_quote(sp_id, sp_quote_type, quote_path);
            if (ret < 0)
                goto out;

            break;
        }

        case 's': { // sign
            if (!data_path) {
                printf("Data path not set\n");
                usage(argv[0]);
                goto out;
            }

            if (!sig_path) {
                printf("Signature path not set\n");
                usage(argv[0]);
                goto out;
            }

            size_t sig_size = 0;
            size_t data_size = 0;
            uint8_t* data = read_file(NULL, data_path, &data_size);
            if (!data)
                goto out;

            ret = load_pod_enclave(enclave_path,
                                   ENCLAVE_DEBUG_ENABLED,
                                   sealed_keys_path,
                                   true, // load sealed state
                                   NULL); // don't export public key
            if (ret < 0)
                goto out;

            // ECALL: get signature size
            sgx_status_t sgx_ret = e_get_signature_size(g_enclave_id, &ret, data, data_size,
                                                        &sig_size);
            if (sgx_ret != SGX_SUCCESS || ret < 0) {
                ret = -1;
                printf("Failed to get signature size\n");
                goto out;
            }

            void* signature = malloc(sig_size);
            if (!signature) {
                ret = -1;
                printf("No memory\n");
                goto out;
            }

            // ECALL: sign data
            sgx_ret = e_sign_data(g_enclave_id, &ret, data, data_size, signature, sig_size);
            if (sgx_ret != SGX_SUCCESS || ret < 0) {
                ret = -1;
                printf("Failed to sign data\n");
                goto out;
            }

            ret = write_file(sig_path, sig_size, signature);
            if (ret < 0)
                goto out;

            printf("Saved signature to '%s'\n", sig_path);
            break;
        }

        default: {
            usage(argv[0]);
            break;
        }
    }

    ret = 0;
out:
    return ret;
}

// OCALL: save sealed enclave state
int o_store_sealed_data(const uint8_t* sealed_data, size_t sealed_size) {
    printf("Saving sealed enclave state to '%s'\n", g_sealed_state_path);
    return write_file(g_sealed_state_path, sealed_size, sealed_data);
}

// OCALL: save enclave's public RSA key
int o_store_public_key(const uint8_t* data, size_t size) {
    printf("Saving public enclave key to '%s'\n", g_public_key_path);
    return write_file(g_public_key_path, size, data);
}

// OCALL: print string
void o_print(const char* str) {
    printf("%s", str);
}
