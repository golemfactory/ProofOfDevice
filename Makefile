targets = all clean

.PHONY: $(targets)
$(targets):
	$(MAKE) -C pod_enclave $@
	$(MAKE) -C pod_app $@
