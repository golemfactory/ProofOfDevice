APP_BIN ?= pod_app
PREFIX ?= /usr/local

ifeq ($(DEBUG), 1)
	CFLAGS := -O0 -g
else
	CFLAGS := -O2
endif

.PHONY: all
all: $(APP_BIN)

CFLAGS += -Wall \
    -Wextra \
    -std=c99 \
    -m64 \
    -D_GNU_SOURCE \
    -I../pod_library

LDFLAGS := -L../pod_library \
           -lpod_sgx

$(APP_BIN): pod_app.o
	$(CC) $^ $(LDFLAGS) -o $@

pod_app.o: pod_app.c
	$(CC) $(CFLAGS) -c $< -o $@

.PHONY: install
install:
	install $(APP_BIN) ${PREFIX}/bin

.PHONY: clean
clean:
	$(RM) *.o $(APP_BIN)
