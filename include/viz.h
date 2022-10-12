#ifndef VIZ_H
#define VIZ_H

#include <math.h>
#include <pthread.h>

#include <SDL2/SDL.h>
#include <SDL2/SDL_ttf.h>
#include <stdbool.h>

#include "tools.h"

static const char *const TTF_FONT_PATH = "/usr/share/fonts/TTF/DejaVuSansMono.ttf";
static const float_t SCALING_LOG_BASE = 10.F;

enum {
    WIDTH = 1500,
    HEIGHT = 600,
    MAX_COLOR_VALUE = 255,
    LABEL_Y_SPACING = 10,
    HUE_OFFSET = 120,
    HUE_MAX_VALUE = 360,
    LABEL_COUNT = 20,
};

typedef struct HSV {
    float_t hue;
    float_t saturation;
    float_t value;
} __attribute__((aligned(16))) HSV;

typedef struct VIZ {
    SDL_Window *window;
    SDL_Renderer *renderer;
    TTF_Font *font;
} __attribute__((aligned(32))) VIZ;

VIZ viz_init(void);

bool viz_loop(void);

void viz_quit(VIZ);

void viz_draw(VIZ viz, float_t *, int32_t);

#endif
