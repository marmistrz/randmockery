#include <unistd.h>
#include <sys/signal.h>

int main() {
    for (;;) {
        kill(getpid(), SIGTRAP);
    }
    return 0;
}
