#ifndef FFTW_H
#define FFTW_H

#include <malloc.h>
#include <math.h>
#include <memory.h>
#include <stdint.h>

#include <fftw3.h>

#include "tools.h"

enum {
    HIGH_CUT = 20000,
    RESULT_MAX = 1000,
    FFTW_DFT_ALIGNMENT __attribute__((unused)) = 32
};

typedef struct FFTW {
    float *signal;
    int signal_size;
    float *result;
    int result_size;
    float rate;
} __attribute__((aligned(FFTW_DFT_ALIGNMENT))) FFTW;

float_t *execute_fftw(FFTW);

#endif
