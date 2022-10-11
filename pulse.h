#include <stdio.h>

#include <pulse/simple.h>
#include <pulse/error.h>

void pulse_init(size_t rate);

void pulse_quit();

float *pulse_read(float *result, size_t size);
