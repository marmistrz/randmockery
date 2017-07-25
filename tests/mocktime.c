#include <time.h>

time_t time(time_t* tloc) {
    static time_t t = 0;
    (void) tloc;
    return t++;
}
