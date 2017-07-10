#include <sys/random.h>
#include <stdint.h>
#include <stdio.h>

int main()
{
	uint64_t x;
	getrandom(&x, sizeof(x), 0);
	printf("Requested %lu bytes, received: %lu\n", sizeof(x), x);
	return 0;
}
