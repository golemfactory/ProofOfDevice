SGX_SDK ?= /opt/intel/sgxsdk
SGX_SSL ?= /opt/intel/sgxssl
SGX_SSL_LIB ?= $(SGX_SSL)/lib64
SGX_INC ?= $(SGX_SDK)/include
SGX_LIB ?= $(SGX_SDK)/lib64

ifeq ($(DEBUG), 1)
	CFLAGS := -O0 -g
else
	CFLAGS := -O2
endif

.PHONY: all
all: pod_app

CFLAGS += -fPIC -Wno-attributes -Wall -Wextra -std=c99 -m64 -D_GNU_SOURCE \
	-I$(SGX_INC) -I../pod_enclave

LDFLAGS := -lcrypto \
	-L$(SGX_LIB) -lsgx_urts -lsgx_uae_service \
	-L$(SGX_SSL_LIB) -lsgx_usgxssl

# Main executable
pod_app: pod_app.o pod_enclave_u.o
	$(CC) $^ $(LDFLAGS) -o $@

pod_app.o: pod_app.c
	$(CC) $(CFLAGS) -c $< -o $@

pod_enclave_u.o: ../pod_enclave/pod_enclave_u.c
	$(CC) $(CFLAGS) -c $< -o $@

.PHONY: clean
clean:
	$(RM) *.o *.so pod_app
