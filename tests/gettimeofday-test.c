#include <time.h>
#include <stdio.h>
#include <unistd.h>
#include <sys/syscall.h>
#include <sys/time.h>

int main() {
    for (int i = 0; i < 10; ++i) {
        // time(NULL) would use vDSO, testing real syscall
        struct timeval tv;
        time_t t = syscall(SYS_gettimeofday, &tv, NULL);
        if (t != 0) return 1;
    }
    return 0;
}
