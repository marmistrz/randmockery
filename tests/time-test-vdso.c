#include <time.h>
#include <stdio.h>
#include <unistd.h>
#include <sys/syscall.h>

int main() {
    for (int i = 0; i < 10; ++i) {
        // time(NULL) would use vDSO, testing real syscall
        time_t t = time(NULL);
        if (t != i) return 1;
    }
    return 0;
}
