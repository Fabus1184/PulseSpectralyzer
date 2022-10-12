#ifndef PULSE_H
#define PULSE_H

#include <math.h>
#include <stdint.h>
#include <stdlib.h>

#include <pulse/error.h>
#include <pulse/simple.h>
#include <pulse/thread-mainloop.h>

#include "tools.h"

enum {
    FRAGSIZE = 5000,
    Pulse_ALIGNMENT __attribute__((unused)) = 8
};

typedef pa_simple *PULSE;

size_t pulse_get_latency(PULSE);

PULSE pulse_init(size_t rate);

void pulse_quit(void *);

float *pulse_read(PULSE, float *, size_t);

#endif
