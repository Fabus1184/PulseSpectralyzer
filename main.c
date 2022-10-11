#include <stdio.h>

#include "fftw.h"
#include "viz.h"
#include "pulse.h"

#define RATE 192000

int main() {
    viz_init();
    pulse_init(RATE);

    const int signal_buffer_size = 4000;
    const int record_buffer_size = signal_buffer_size / 2;
    const int draw_buffer_size = 20000;
    float *signal_buffer = malloc(signal_buffer_size * sizeof(float));
    float *record_buffer = malloc(record_buffer_size * sizeof(float));
    float *draw_buffer = malloc(draw_buffer_size * sizeof(float));
    printf("Latency: %0.3fs\n", (float) signal_buffer_size / (float) RATE);

    while (!viz_loop()) {
        memmove(signal_buffer + record_buffer_size, signal_buffer,
                (signal_buffer_size - record_buffer_size) * sizeof(float));
        pulse_read(record_buffer, record_buffer_size);
        memcpy(signal_buffer, record_buffer, record_buffer_size * sizeof(float));
        execute_fftw(signal_buffer, signal_buffer_size, draw_buffer, draw_buffer_size, RATE);
        viz_draw(draw_buffer, draw_buffer_size);
    }

    pulse_quit();
    viz_quit();

    free(signal_buffer);
    free(record_buffer);
    free(draw_buffer);
    return 0;
}
