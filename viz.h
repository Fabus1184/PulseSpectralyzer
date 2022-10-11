#include <pthread.h>
#include <math.h>

#include <SDL2/SDL.h>
#include <SDL2/SDL_ttf.h>

void viz_init();

int viz_loop();

void viz_quit();

void viz_draw(const float *data, int size);
