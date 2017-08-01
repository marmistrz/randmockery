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
    int ret = 0;
    #pragma omp parallel
    {
        uint32_t x = VAL;
        syscall(SYS_getrandom, &x, sizeof(x), 0);
        if (x != 0) {
            printf("Thread %u: incorrect x: %u", omp_get_thread_num(), x);
            #pragma omp atomic write
            ret = 1;
        }
    }

    return ret;
}
