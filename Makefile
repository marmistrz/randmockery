CFLAGS ?= -Wall -Wextra
all: getrandom getrandom-test
clean:
	rm -f getrandom
.PHONY: clean all