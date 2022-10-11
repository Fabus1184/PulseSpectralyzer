#include "fftw.h"

#define HIGH_CUT 20000

float *execute_fftw(float *signal, int signal_size, float *result, int result_size, float rate) {
    fftwf_complex *out = fftwf_alloc_complex(1 + (signal_size / 2));
    fftwf_plan plan = fftwf_plan_dft_r2c_1d((int) signal_size, signal, out, FFTW_ESTIMATE);
    fftwf_execute(plan);

    for (int i = 0; i < 1 + (signal_size / 2); i++) {
        out[i][0] /= (float) sqrt(signal_size);
        out[i][1] /= (float) sqrt(signal_size);
    }

    float high_cut_index = HIGH_CUT / rate * (float) signal_size;

    for (int i = 0; i < result_size; i++) {
        float index = (float) i / (float) result_size * (float) (high_cut_index);
        int index_floor = (int) index;
        int index_ceil = index_floor + 1;
        float index_fraction = index - (float) index_floor;
        float value_floor = sqrtf(
                out[index_floor][0] * out[index_floor][0] + out[index_floor][1] * out[index_floor][1]);
        float value_ceil = sqrtf(out[index_ceil][0] * out[index_ceil][0] + out[index_ceil][1] * out[index_ceil][1]);
        result[i] = value_floor + (value_ceil - value_floor) * index_fraction;
    }

    // scale the result logarithmically
    for (int i = 0; i < result_size; i++) {
        result[i] = log10f(result[i] + 1);
    }


    fftwf_destroy_plan(plan);
    fftwf_free(out);
}
