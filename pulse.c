#include "pulse.h"

pa_sample_spec SS;
pa_simple *S;

void pulse_init(size_t rate) {
    SS = (pa_sample_spec) {
            .format = PA_SAMPLE_FLOAT32,
            .channels = 1,
            .rate = rate,
    };

    pa_buffer_attr attr = {
            .maxlength = -1,
            .tlength = -1,
            .prebuf = -1,
            .minreq = -1,
            .fragsize = 5000,
    };

    int error;
    S = pa_simple_new(NULL, "PulseVizCli", PA_STREAM_RECORD, NULL, "Music", &SS, NULL, &attr, &error);
    if (!S) {
        fprintf(stderr, "Error creating stream: %s\n", pa_strerror(error));
    }
}

void pulse_quit() {
    pa_simple_free(S);
}

float *pulse_read(float *result, size_t size) {
    int error;
    if (pa_simple_read(S, result, size * sizeof(float), &error) < 0) {
        fprintf(stderr, "Error reading %ld bytes: %s\n", size * sizeof(float), pa_strerror(error));
    }
}
