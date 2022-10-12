#include "../include/fftw.h"

float_t *execute_fftw(FFTW dft) {
    fftwf_complex *out = fftwf_alloc_complex(1 + ((size_t) dft.signal_size / 2));
    fftwf_plan plan = fftwf_plan_dft_r2c_1d(dft.signal_size, dft.signal, out, FFTW_ESTIMATE);
    fftwf_execute(plan);

    memset(dft.result, 0, (size_t) dft.result_size * sizeof(float));
    for (int32_t i = 0; i < dft.result_size; ++i) {
        float_t frequency = HIGH_CUT * ((float_t) i / (float_t) dft.result_size);
        int32_t index = ROUND(frequency / dft.rate * (float) dft.signal_size);
        dft.result[i] += sqrtf((out[index][0] * out[index][0]) + (out[index][1] * out[index][1]));
    }

    for (int32_t i = 0; i < dft.result_size; ++i) {
        dft.result[i] /= (float_t) dft.signal_size / (float_t) dft.result_size;
    }

    for (int32_t i = 0; i < dft.result_size; ++i) {
        dft.result[i] /= RESULT_MAX;
    }

    fftwf_destroy_plan(plan);
    fftwf_free(out);
}
