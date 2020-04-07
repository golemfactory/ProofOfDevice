#[repr(C)]
pub(crate) struct IasContext {
    _private: [u8; 0],
}

#[link(name = "sgx_util")]
extern "C" {
    // ias.h
    pub(crate) fn ias_init(
        ias_api_key: *const libc::c_char,
        ias_verify_url: *const libc::c_char,
        ias_sigrl_url: *const libc::c_char,
    ) -> *mut IasContext;
    pub(crate) fn ias_cleanup(context: *mut IasContext);
    pub(crate) fn ias_get_sigrl(
        context: *mut IasContext,
        gid: *const u8,
        sigrl_size: *mut usize,
        sigrl: *mut *mut libc::c_void,
    ) -> libc::c_int;
    pub(crate) fn ias_verify_quote(
        context: *mut IasContext,
        quote: *const libc::c_void,
        quote_size: usize,
        nonce: *const libc::c_char,
        report_path: *const libc::c_char,
        sig_path: *const libc::c_char,
        cert_path: *const libc::c_char,
        advisory_path: *const libc::c_char,
    ) -> libc::c_int;
    // attestation.h
    pub(crate) fn display_quote(quote_data: *const libc::c_void, quote_size: usize);
    pub(crate) fn verify_ias_report(
        ias_report: *const u8,
        ias_report_size: usize,
        ias_sig_b64: *mut u8,
        ias_sig_64_size: usize,
        allow_outdated_tcb: bool,
        nonce: *const libc::c_char,
        mr_signer: *const libc::c_char,
        mr_enclave: *const libc::c_char,
        isv_prod_id: *const libc::c_char,
        isv_svn: *const libc::c_char,
        report_data: *const libc::c_char,
        ias_pub_key_pem: *const libc::c_char,
    ) -> libc::c_int;
    pub(crate) fn verify_quote(
        quote_data: *const libc::c_void,
        quote_size: usize,
        mr_signer: *const libc::c_char,
        mr_enclave: *const libc::c_char,
        isv_prod_id: *const libc::c_char,
        isv_svn: *const libc::c_char,
        report_data: *const libc::c_char,
    ) -> libc::c_int;
}
