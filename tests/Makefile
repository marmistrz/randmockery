CFLAGS ?= -Wall -Wextra -O2
BINS = getrandom-test getrandom-test-mocked time-test time-test-vdso clock_gettime-test
all: $(BINS) libmocktime.so
clean:
	rm -f $(BINS)
libmocktime.so: libmocktime.o
	$(CC) $(CFLAGS) -shared -o $@ $^
libmocktime.o: mocktime.c
	$(CC) $(CFLAGS) -fPIC -c $^ -o $@

.PHONY: clean all
