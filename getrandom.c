#include <sys/random.h>
#include <stdint.h>
#include <stdio.h>

int main()
{
	uint64_t x;
	getrandom(&x, sizeof(x), 0);
	printf("issued getrandom(%lu, %lu, %d)\n", (uint64_t) &x, sizeof(x), 0);
	printf("received: %lu\n", x);
	uint8_t* xptr = (uint8_t*) &x;
	printf("lowest byte is %d\n", *xptr);
	return 0;
}
