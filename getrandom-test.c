#include <sys/random.h>
#include <stdint.h>
#include <stdio.h>
#include <assert.h>

#define VAL 42424242

int main()
{
	uint32_t x = VAL;
	uint32_t y = VAL;
	getrandom(&x, sizeof(x), 0);
	assert(x == 0);
	assert(y == VAL);
	return 0;
}
