#ifndef POD_CONFIG_H
#define POD_CONFIG_H

/** Default file name to save sealed keys to. */
#define DEFAULT_SEALED_KEYS_PATH "pod_data.sealed"

/** Default path to enclave binary. */
#define DEFAULT_ENCLAVE_PATH "pod_enclave.signed.so"

/** Default file name to save enclave quote to. */
#define DEFAULT_ENCLAVE_QUOTE_PATH "pod.quote"

/** Maximum quote size. */
#define MAX_QUOTE_SIZE 2048

/** Maximum sealed state size. */
#define MAX_SEALED_STATE_SIZE 4096

/** Enables enclave debugging and NULLIFIES ENCLAVE MEMORY PROTECTION. */
#define ENCLAVE_DEBUG_ENABLED 1

#endif /* POD_CONFIG_H */
