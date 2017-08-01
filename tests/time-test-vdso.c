#include <stdio.h>
#include <sys/syscall.h>
#include <time.h>
#include <unistd.h>

int main() {
    for (int i = 0; i < 10; ++i) {
        // time(NULL) would use vDSO, testing real syscall
        time_t t = time(NULL);
        if (t != i) {
            printf("Incorrect time! Expected: %d, got: %ld", i, t);
            return 1;
        }
    }
    return 0;
}
