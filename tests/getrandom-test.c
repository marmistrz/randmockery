#include <stdint.h>
#include <assert.h>

#include <unistd.h>
#include <sys/syscall.h>

#define VAL 42424242

// older glibc doesn't have getrandom(2)
#ifndef SYS_getrandom
#define SYS_getrandom 318
#endif

int main()
{
	uint32_t x = VAL;
	uint32_t y = VAL;
	syscall(SYS_getrandom, &x, sizeof(x), 0);
	printf("%d %d", x, y);
	if (x == 0 && y == VAL) return 0;
	else return 1;
}
