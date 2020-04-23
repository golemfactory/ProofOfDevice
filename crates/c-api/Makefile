targets = all clean install

.PHONY: $(targets)
$(targets):
	$(MAKE) -C pod-enclave $@
	$(MAKE) -C pod-lib $@
