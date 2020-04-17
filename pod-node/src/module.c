#include <node_api.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "pod_sgx.h"

#define ENCLAVE_PATH "../pod-client/pod_enclave/pod_enclave.signed.so"
#define SEALED_KEYS_PATH "pod_data.sealed"
#define PUBLIC_KEY_PATH "pod_pubkey"
#define ENCLAVE_QUOTE_PATH "pod.quote"
#define QUOTE_TYPE "u"
#define QUOTE_BUFFER_SIZE 4096

void FreeQuote(napi_env env, void* nativeObject, void* finalize_hint) {
  free((char*)nativeObject);
}

napi_value Register(napi_env env, napi_callback_info info) {
  napi_status status;
  // expected argument size
  size_t argc = 2;
  napi_value argv[2];

  // fetch arguments as an array
  status = napi_get_cb_info(env, info, &argc, argv, NULL, NULL);

  if (status != napi_ok) {
    napi_throw_error(env, NULL, "Failed to parse arguments");
  }

  // convert arguments to C values
  // the SPID value is 32bytes long, however, since strings in C
  // are nul terminated we actually need 33bytes
  char spid[33];
  size_t v1;
  status = napi_get_value_string_utf8(env, argv[0], &spid, 33, &v1);

  if (status != napi_ok) {
    napi_throw_error(env, NULL, "Invalid SPID was passed as argument");
  }

  char username[33];
  size_t v2;
  status = napi_get_value_string_utf8(env, argv[1], &username, 33, &v2);

  if (status != napi_ok) {
    napi_throw_error(env, NULL, "Invalid SPID was passed as argument");
  }

  int ret = -1;
  ret = pod_init_enclave(ENCLAVE_PATH, spid, QUOTE_TYPE, SEALED_KEYS_PATH, PUBLIC_KEY_PATH, ENCLAVE_QUOTE_PATH);
  // ret = 0; // mock 

  if(ret < 0) {
  	napi_throw_error(env, NULL, "Initialization failed");
  }
  pod_unload_enclave();

  // OK, so in order to pass a foreign array into JS space, we need to
  // allocate on the heap, pass in a pointer wrapped inside a JS struct,
  // and then handle a finalize callback to free in native so we don't
  // leak.
  char* quote = (char *) malloc(QUOTE_BUFFER_SIZE);
  size_t quote_size = 0;
  read_file(quote, ENCLAVE_QUOTE_PATH, &quote_size);

  napi_value nQuote;
  status = napi_create_external_buffer(env, quote_size, quote, FreeQuote, NULL, &nQuote);

  if (status != napi_ok) {
    napi_throw_error(env, NULL, "Unable to create return value");
  }

  return nQuote;
}

napi_value Init(napi_env env, napi_value exports) {
  napi_status status;
  napi_value fn;

  status = napi_create_function(env, NULL, 0, Register, NULL, &fn);
  if (status != napi_ok) {
    napi_throw_error(env, NULL, "Unable to wrap native function");
  }

  status = napi_set_named_property(env, exports, "register", fn);
  if (status != napi_ok) {
    napi_throw_error(env, NULL, "Unable to populate exports");
  }

  return exports;
}

NAPI_MODULE(NODE_GYP_MODULE_NAME, Init)
