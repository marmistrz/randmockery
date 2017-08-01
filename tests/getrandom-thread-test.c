#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>

#include <sys/syscall.h>
#include <unistd.h>

#include <omp.h>

#define VAL 42424242

// older glibc doesn't have getrandom(2)
#ifndef SYS_getrandom
#define SYS_getrandom 318
#endif

int main() {
#pragma omp parallel
    {
        uint32_t x = VAL;
        syscall(SYS_getrandom, &x, sizeof(x), 0);
        printf("%u %u\n", omp_get_thread_num(), x);
    }

    return 0;
}
