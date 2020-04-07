#include <getopt.h>
#include <stdbool.h>
#include <stdio.h>

#include "pod_app.h"
#include "pod_sgx.h"

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

int main(int argc, char* argv[]) {
    int this_option = 0;
    char* sp_id = NULL;
    char* sp_quote_type = NULL;
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
                return 0;
            case 's':
                sealed_keys_path = optarg;
                break;
            case 'e':
                enclave_path = optarg;
                break;
            case 'p':
                public_key_path = optarg;
                break;
            case 'i':
                sp_id = optarg;
                break;
            case 't':
                sp_quote_type = optarg;
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
            if (!sp_id) {
                printf("SPID not set\n");
                usage(argv[0]);
                goto out;
            }

            if (!sp_quote_type) {
                printf("Quote type not set\n");
                usage(argv[0]);
                goto out;
            }

            ret = pod_init_enclave(enclave_path, sp_id, sp_quote_type, sealed_keys_path,
                                   public_key_path, quote_path);
            if (ret < 0)
                goto out;

            ret = pod_unload_enclave();
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

            ret = pod_load_enclave(enclave_path, sealed_keys_path);
            if (ret < 0)
                goto out;

            ret = pod_sign_file(data_path, sig_path);
            if (ret < 0)
                goto out;

            ret = pod_unload_enclave();
            break;
        }

        default: {
            usage(argv[0]);
            ret = 0;
            break;
        }
    }

out:
    return ret;
}
