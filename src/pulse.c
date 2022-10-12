#include "../include/pulse.h"

PULSE pulse_init(size_t rate) {
    pa_sample_spec sample_spec = (pa_sample_spec) {
            .format = PA_SAMPLE_FLOAT32,
            .channels = 1,
            .rate = (uint32_t) rate,
    };

    pa_buffer_attr attributes = {
            .maxlength = (uint32_t) -1,
            .tlength = (uint32_t) -1,
            .prebuf = (uint32_t) -1,
            .minreq = (uint32_t) -1,
            .fragsize = FRAGSIZE,
    };

    int32_t error[1];
    pa_simple *simple = pa_simple_new(NULL, "PulseSpectralyzer", PA_STREAM_RECORD, NULL, "Music", &sample_spec, NULL,
                                      &attributes, error);
    CHECK(simple == NULL, pa_strerror, *error)
    return (PULSE) {simple};
}

size_t pulse_get_latency(PULSE pulse) {
    int32_t error[1] = {0};
    size_t latency = pa_simple_get_latency(pulse, error);
    CHECK(*error != 0, pa_strerror, *error)
    return latency;
}

void pulse_quit(void *pulse) {
    pa_simple_free(*(PULSE *) pulse);
}

float_t *pulse_read(PULSE pulse, float_t *result, size_t size) {
    int32_t error[1];
    CHECK((pa_simple_read(pulse, result, size * sizeof(float), error) < 0), pa_strerror, *error)
}
