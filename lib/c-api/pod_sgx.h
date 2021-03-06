#ifndef POD_SGX_H
#define POD_SGX_H

/** Enables enclave debugging and NULLIFIES ENCLAVE MEMORY PROTECTION. */
#define ENCLAVE_DEBUG_ENABLED 1

/*!
 *  \brief Get size of an open file.
 *
 *  \param[in] fd Open file descriptor.
 *
 *  \return File size or -1 on error.
 */
ssize_t get_file_size(int fd);

/*!
 *  \brief Read file contents to buffer.
 *
 *  \param[in]     buffer Buffer to read data to. If NULL, this function allocates one.
 *  \param[in]     path   Path to the file.
 *  \param[in,out] size   On entry, number of bytes to read. 0 means to read the entire file.
 *                        On exit, number of bytes read.
 *
 *  \return On success, pointer to the data buffer. If \p buffer was NULL, caller should free this.
 *          On failure, NULL.
 */
void* read_file(void* buffer, const char* path, size_t* size);

/*!
 *  \brief Write buffer to file.
 *
 *  \param[in] path   File path.
 *  \param[in] size   \p buffer size.
 *  \param[in] buffer Buffer to write data from.
 *
 *  \return 0 on success, errno on error.
 */
int write_file(const char* path, size_t size, const void* buffer);

/*!
 *  \brief Initialize PoD enclave.
 *         Loads enclave, generates new enclave key pair, and seals the private key into the provided
 *         buffer.
 *
 *  \param[in] enclave_path       Path to enclave binary.
 *  \param[in] sealed_state       Buffer to seal the private key to.
 *  \param[in] sealed_state_size  Size of the provided buffer.
 *
 *  \return On success, number of bytes written to the buffer. On failure, negative value.
 */
int pod_init_enclave(const char* enclave_path, uint8_t* sealed_state, size_t sealed_state_size);

/*!
 *  \brief Load PoD enclave and restore its private key from sealed state.
 *
 *  \param[in] enclave_path      Path to enclave binary.
 *  \param[in] sealed_state_path Path to sealed enclave state.
 *
 *  \return 0 on success, negative on error.
 */
int pod_load_enclave(const char* enclave_path, const uint8_t* sealed_state, size_t sealed_state_size);

/*!
 *  \brief Unload PoD enclave.
 *
 *  \return 0 on success, negative on error.
 */
int pod_unload_enclave(void);

/*!
 *  \brief Generate valid quote of this PoD enclave for remote attestation with IAS services.
 *
 *  \param[in] sp_id_str          Service Provider ID (hex string).
 *  \param[in] sp_quote_type_str  Quote type as string ("linkable"/"unlinkable").
 *  \param[in] quote_buffer       Buffer to save the quote to.
 *  \param[in] quote_buffer_size  Size of the provided buffer.
 *
 *  \return On success, number of bytes written to the buffer. On failure, negative value.
 */
int pod_get_quote(const char* sp_id_str, const char* sp_quote_type_str, uint8_t* quote_buffer,
                  size_t quote_buffer_size);

/*!
 *  \brief Create PoD enclave digital signature for data buffer.
 *
 *  \param[in]  data           Buffer with data to sign.
 *  \param[in]  data_size      Size of \p data in bytes.
 *  \param[out] signature      Buffer that will receive the signature.
 *  \param[in]  signature_size Size of \p signature in bytes.
 *
 *  \return 0 on success, negative on error.
 */
int pod_sign_buffer(const void* data, size_t data_size, void* signature, size_t signature_size);

/*!
 *  \brief Create PoD enclave digital signature for a file.
 *
 *  \param[in] input_path     Path to file to sign.
 *  \param[in] signature_path Path where the signature will be saved.
 *
 *  \return 0 on success, negative on error.
 */
int pod_sign_file(const char* input_path, const char* signature_path);

#endif /* POD_SGX_H */
