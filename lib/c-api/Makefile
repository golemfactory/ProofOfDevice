SGX_SDK ?= /opt/intel/sgxsdk
SGX_SSL ?= /opt/intel/sgxssl
SGX_SSL_LIB ?= $(SGX_SSL)/lib64
SGX_INC ?= $(SGX_SDK)/include
SGX_LIB ?= $(SGX_SDK)/lib64
PREFIX ?= /usr/local
LIBRARY_BIN ?= libpod_sgx.so

ifeq ($(DEBUG), 1)
    CFLAGS := -O0 -g
else
    CFLAGS := -O2
endif

.PHONY: all
all: $(LIBRARY_BIN)

CFLAGS += -fPIC \
    -Wno-attributes \
    -Wall \
    -Wextra \
    -std=c99 \
    -m64 \
    -D_GNU_SOURCE \
    -I$(SGX_INC) \
    -I../../pod-enclave 

LDFLAGS := -lcrypto \
    -L$(SGX_LIB) -lsgx_urts -lsgx_uae_service \
    -L$(SGX_SSL_LIB) -lsgx_usgxssl \
    -shared

# Main binary
$(LIBRARY_BIN): pod_sgx.o pod_log.o pod_enclave_u.o
	$(CC) $^ $(LDFLAGS) -o $@

pod_sgx.o: pod_sgx.c
	$(CC) $(CFLAGS) -c $< -o $@

pod_log.o: pod_log.c
	$(CC) $(CFLAGS) -c $< -o $@

pod_enclave_u.o: ../../pod-enclave/pod_enclave_u.c
	$(CC) $(CFLAGS) -c $< -o $@

.PHONY: install
install:
	install $(LIBRARY_BIN) ${PREFIX}/lib
	ldconfig

.PHONY: clean
clean:
	$(RM) *.o *.so
