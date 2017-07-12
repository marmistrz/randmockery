#include <assert.h>
#include <stdint.h>
#include <string.h>

#include <sys/syscall.h>
#include <unistd.h>

// older glibc doesn't have getrandom(2)
#ifndef SYS_getrandom
#define SYS_getrandom 318
#endif

uint16_t expected[] = {
    0xcd32, 0x7a6f, 0x6ebc, 0x0ad9, 0x9c58, 0xfc15, 0x4a60, 0x7362,
    0xd2d1, 0x6076, 0xc2ed, 0xff7c, 0x3e3a, 0x4c60, 0x48bc, 0x26cb,
    0x5da8, 0xc0aa, 0x4bd3, 0x678d, 0x7608, 0xc54a, 0x9af9, 0x90d7,
    0x2225, 0x7ac5, 0x6488, 0x5f70, 0xb5bf, 0x1493, 0xb081, 0x7c2d,
};

int main() {
    const size_t N = 32;
    uint16_t bytes[N];
    syscall(SYS_getrandom, bytes, sizeof(bytes), 0);
    return memcmp(expected, bytes, N) == 0 ? 0 : 1;
}
