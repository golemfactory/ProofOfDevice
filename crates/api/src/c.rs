#[link(name = "pod_sgx")]
extern "C" {
    pub(super) fn pod_init_enclave(
        enclave_path: *const libc::c_char,
        sealed_keys: *mut u8,
        sealed_keys_size: usize,
    ) -> libc::c_int;
    pub(super) fn pod_load_enclave(
        enclave_path: *const libc::c_char,
        sealed_keys: *const u8,
        sealed_keys_size: usize,
    ) -> libc::c_int;
    pub(super) fn pod_unload_enclave() -> libc::c_int;
    pub(super) fn pod_get_quote(
        sp_id_str: *const libc::c_char,
        sp_quote_type_str: *const libc::c_char,
        quote_buffer: *mut u8,
        quote_buffer_size: usize,
    ) -> libc::c_int;
    pub(super) fn pod_sign_buffer(
        data: *const libc::c_void,
        data_size: usize,
        signature: *mut libc::c_void,
        signature_size: usize,
    ) -> libc::c_int;
}
