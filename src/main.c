#include <pthread.h>

#include "../include/fftw.h"
#include "../include/pulse.h"
#include "../include/viz.h"

#define DURATION 0.03F
#define RECORD_BUFFER_SIZE (ROUND(DURATION * RATE))

enum {
    RATE = 192000,
    DRAW_BUFFER_SIZE = 20000,
    THOUSAND = 1000,
    PULSE_THREAD_ALIGNMENT __attribute__((unused)) = 32
};

typedef struct Pulse_thread_args {
    float_t *record_buffer;
    float_t *draw_buffer;
    pthread_mutex_t *mutex;
    sig_atomic_t *update;
} __attribute__((aligned(PULSE_THREAD_ALIGNMENT))) Pulse_thread_args;

_Noreturn void *pulse_thread(void *thread_args) {
    Pulse_thread_args *args = (Pulse_thread_args *) thread_args;

    pthread_setcanceltype(PTHREAD_CANCEL_DEFERRED, NULL);

    PULSE pulse = pulse_init(RATE);
    size_t latency = pulse_get_latency(pulse);
    printf("Buffer size: %d, Latency: ~%0.3fms\n", RECORD_BUFFER_SIZE,
           ((float_t) latency / THOUSAND) + (DURATION * THOUSAND));
    pthread_cleanup_push(pulse_quit, pulse) while (1) {
        pulse_read(pulse, args->record_buffer, (size_t) RECORD_BUFFER_SIZE);
        pthread_mutex_lock(args->mutex);
        execute_fftw(
                (FFTW){args->record_buffer, RECORD_BUFFER_SIZE, args->draw_buffer, DRAW_BUFFER_SIZE,
                       RATE});
        *args->update = 1;
        pthread_mutex_unlock(args->mutex);
    }
    pthread_cleanup_pop(1);
}

int main(void) {
    pthread_mutex_t mutex = PTHREAD_MUTEX_INITIALIZER;
    sig_atomic_t update = 1;
    float_t record_buffer[RECORD_BUFFER_SIZE];
    float_t draw_buffer[DRAW_BUFFER_SIZE];
    memset(draw_buffer, 0, DRAW_BUFFER_SIZE * sizeof(float));


    pthread_t pid[1];
    Pulse_thread_args args = {record_buffer, draw_buffer, &mutex, &update};
    pthread_create(pid, NULL, pulse_thread, &args);

    VIZ viz = viz_init();

    while (!viz_loop()) {
        if (1 == update) {
            pthread_mutex_lock(&mutex);
            viz_draw(viz, draw_buffer, DRAW_BUFFER_SIZE);
            update = 0;
            pthread_mutex_unlock(&mutex);
        }
    }
    pthread_kill(*pid, SIGKILL);
    pthread_cancel(*pid);
    pthread_join(*pid, NULL);
    viz_quit(viz);

    return 0;
}
