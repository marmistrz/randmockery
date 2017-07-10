CFLAGS ?= -Wall -Wextra
all: getrandom getrandom-test
clean:
	rm -f getrandom getrandom-test
.PHONY: clean all