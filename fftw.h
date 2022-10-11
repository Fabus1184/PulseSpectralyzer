#include <math.h>
#include <memory.h>
#include <malloc.h>

#include <fftw3.h>

float *execute_fftw(float *signal, int signal_size, float *result, int result_size, float rate);
