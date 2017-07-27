#include <time.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/syscall.h>

void check_clock(clockid_t clk)
{
    struct timespec t;
    syscall(SYS_clock_gettime, clk, &t);

    if (t.tv_sec != 0 || t.tv_nsec != 0) {
        printf("%ld %ld\n", t.tv_sec, t.tv_nsec);
        exit(EXIT_FAILURE);
    }
}

int main() {
    check_clock(CLOCK_REALTIME);
    check_clock(CLOCK_MONOTONIC);
    check_clock(CLOCK_BOOTTIME);
    return 0;
}
