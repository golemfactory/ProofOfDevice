SGX_SDK ?= /opt/intel/sgxsdk
SGX_SSL ?= /opt/intel/sgxssl
SGX_SSL_INC ?= $(SGX_SSL)/include
SGX_SSL_LIB ?= $(SGX_SSL)/lib64
SGX_INC ?= $(SGX_SDK)/include
SGX_LIB ?= $(SGX_SDK)/lib64
ENCLAVE_CONFIG ?= pod_enclave.config.xml
ENCLAVE_SIGNING_PUBLIC_KEY ?= pod_signing_pubkey.pem
ENCLAVE_BINARY ?= pod_enclave.so
ENCLAVE_BINARY_SIGNED ?= pod_enclave.signed.so

ifeq ($(DEBUG), 1)
	CFLAGS += -O0 -g
else
	CFLAGS += -O2
endif

SGX_EDGER8R := $(SGX_SDK)/bin/x64/sgx_edger8r
SGX_SIGNER := $(SGX_SDK)/bin/x64/sgx_sign
EDGER8R_FLAGS := --search-path $(SGX_INC) --search-path $(SGX_SSL_INC)
MRSIGNER := pod_enclave.mrsigner
MRSIGNER_H := pod_enclave_mrsigner.h

.PHONY: all
all: $(ENCLAVE_BINARY_SIGNED)

.PHONY: check_key
check_key:
ifndef ENCLAVE_SIGNING_KEY
	$(error ENCLAVE_SIGNING_KEY not set)
endif

$(ENCLAVE_SIGNING_PUBLIC_KEY): check_key $(ENCLAVE_SIGNING_KEY)
	openssl rsa -in $(ENCLAVE_SIGNING_KEY) -pubout -out $(ENCLAVE_SIGNING_PUBLIC_KEY)

# PoD enclave
CFLAGS += -m64 \
	-Wall \
	-Wextra \
    -D_GNU_SOURCE \
	-nostdinc \
	-fvisibility=hidden \
	-fpie \
	-fstack-protector \
	-std=c99 \
	-I$(SGX_INC) \
	-I$(SGX_INC)/tlibc \
	-I$(SGX_SSL_INC)

LDFLAGS := -Wl,--no-undefined -nostdlib -nodefaultlibs -nostartfiles \
	-L$(SGX_LIB) -L$(SGX_SSL_LIB) \
	-Wl,--whole-archive -lsgx_tsgxssl -Wl,--no-whole-archive -lsgx_tsgxssl_crypto \
	-Wl,--whole-archive -lsgx_trts -Wl,--no-whole-archive \
	-Wl,--start-group -lsgx_tstdc -lsgx_tcxx -lsgx_tcrypto -lsgx_tservice -Wl,--end-group \
	-Wl,-Bstatic -Wl,-Bsymbolic -Wl,--no-undefined \
	-Wl,-pie,-eenclave_entry -Wl,--export-dynamic  \
	-Wl,--defsym,__ImageBase=0

$(ENCLAVE_BINARY_SIGNED): $(ENCLAVE_BINARY) $(ENCLAVE_SIGNING_KEY) $(ENCLAVE_SIGNING_PUBLIC_KEY)
	$(SGX_SIGNER) gendata -enclave $< -out $<.sigdata -config $(ENCLAVE_CONFIG) -ignore-init-sec-error
	# replace the command below to use HSM for signing operation
	openssl dgst -sha256 -out $<.sig -sign $(ENCLAVE_SIGNING_KEY) $<.sigdata
	$(SGX_SIGNER) catsig -enclave $< -sig $<.sig \
		-unsigned $<.sigdata -key $(ENCLAVE_SIGNING_PUBLIC_KEY) -out $@ \
		-config $(ENCLAVE_CONFIG) -ignore-init-sec-error
	$(SGX_SIGNER) dump -enclave $@ -dumpfile enclave.meta -ignore-init-sec-error

$(MRSIGNER): $(ENCLAVE_SIGNING_PUBLIC_KEY) openssl_parse.awk
	openssl rsa -in $(ENCLAVE_SIGNING_PUBLIC_KEY) -pubin -text -noout \
		| awk -f openssl_parse.awk | xxd -p -r | sha256sum -b \
		| xxd -r -ps -l 32 > $@

# header with mrsigner value
$(MRSIGNER_H): $(MRSIGNER)
	xxd -i -c 8 $(MRSIGNER) > $@

$(ENCLAVE_BINARY): pod_enclave_t.o pod_enclave.o
	$(CC) $^ $(LDFLAGS) -o $@

pod_enclave.o: pod_enclave.c $(MRSIGNER_H) pod_enclave_u.c
	$(CC) $(CFLAGS) -c $< -o $@

pod_enclave_t.o: pod_enclave_t.c
	$(CC) $(CFLAGS) -c $< -o $@

# ECALL/OCALL interface (generated files)
pod_enclave_t.c: pod_enclave.edl
	$(SGX_EDGER8R) $(EDGER8R_FLAGS) --trusted $<

pod_enclave_u.c: pod_enclave.edl
	$(SGX_EDGER8R) $(EDGER8R_FLAGS) --untrusted $<

.PHONY: install
install: ;

.PHONY: clean
clean:
	$(RM) *.o *.so
	$(RM) pod_enclave_t.c pod_enclave_t.h pod_enclave_u.c pod_enclave_u.h
	$(RM) *.meta *.mrsigner *_mrsigner.h *.sig *.sigdata $(ENCLAVE_SIGNING_PUBLIC_KEY)
