{
  "targets": [
    {
      "target_name": "module",
      "sources": [ "./src/module.c" ],
      "include_dirs": [ "../pod-client/pod_library/" ],
      "link_settings": {
        "libraries": [ "-lpod_sgx" ]
      }
    }
  ]
}
