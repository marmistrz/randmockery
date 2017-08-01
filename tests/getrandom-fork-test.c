#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdnoreturn.h>
#include <sys/wait.h>

#include <sys/socket.h>
#include <sys/syscall.h>
#include <unistd.h>

#define VAL 42424242

// older glibc doesn't have getrandom(2)
#ifndef SYS_getrandom
#define SYS_getrandom 318
#endif

void chkerr(int64_t ret, const char* const desc) {
    if (ret < 0) {
        perror(desc);
        exit(EXIT_FAILURE);
    }
}

void sys_chkerr(int64_t ret, const char* const desc) {
    if (ret < 0) {
        errno = (int) -ret;
        perror(desc);
        exit(EXIT_FAILURE);
    }
}

const uint32_t CHILD_X_EXP = 2054147378;
const uint32_t X_EXP = 182021820;

int main() {
    uint32_t x = VAL;
    int ipc_pipe[2];

    chkerr(pipe(ipc_pipe), "pipe");
    int read_pipe = ipc_pipe[0];
    int write_pipe = ipc_pipe[1];

    pid_t pid = fork();
    chkerr(pid, "fork");
    if (pid == 0) {
        sys_chkerr(syscall(SYS_getrandom, &x, sizeof(x), 0), "syscall");
        if (x != CHILD_X_EXP) printf("CHILD: bad x: %u\n", x);
        chkerr(write(write_pipe, &x, sizeof(x)), "write");
        return 0;
    } else {
        uint32_t child_x;
        chkerr(read(read_pipe, &child_x, sizeof(child_x)), "read");

        // The order is important here. If the parent process calls getrandom(2)
        // after he reads from the pipe, the order of calls is deterministic.
        // Otherwise the numbers generated in the two process may be swapped
        // since the parent process is sequential
        sys_chkerr(syscall(SYS_getrandom, &x, sizeof(x), 0), "syscall");
        // Wait for the child process to exit. If we don't, tests will start to
        // fail mysteriously
        chkerr(wait(NULL), "wait");
        if (x != X_EXP || child_x != CHILD_X_EXP) {
            printf("Bad x computed:\n"
                   "parent: %d, child %d\n",
                   x, child_x);
            printf("Expected:\n"
                   "parent: %d, child: %d",
                   X_EXP, CHILD_X_EXP);
            return 1;
        }
        return 0;
    }
}
