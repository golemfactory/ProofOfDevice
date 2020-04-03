targets = all clean install

.PHONY: $(targets)
$(targets):
	$(MAKE) -C pod_enclave $@
	$(MAKE) -C pod_library $@
	$(MAKE) -C pod_app $@
