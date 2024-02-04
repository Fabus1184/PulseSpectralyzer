#include "../include/viz.h"

float_t hsv_helper(HSV color, float_t n) {
    float_t temp = fmodf(n + (color.hue / 60.F), 6.F);
    return color.value * (1 - (color.saturation * fmaxf(fminf(fminf(temp, 4 - temp), 1), 0)));
}

SDL_Color hsv_to_rgb(HSV color) {
    return (SDL_Color){
            (uint8_t) ROUND(hsv_helper(color, 5) * MAX_COLOR_VALUE),
            (uint8_t) ROUND(hsv_helper(color, 3) * MAX_COLOR_VALUE),
            (uint8_t) ROUND(hsv_helper(color, 1) * MAX_COLOR_VALUE),
            MAX_COLOR_VALUE};
}

bool viz_loop(void) {
    SDL_Event event;
    return (1 == SDL_PollEvent(&event)) &&
           (((event.type == SDL_KEYDOWN) && (event.key.keysym.sym == SDLK_q)) || (event.type == SDL_QUIT));
}

int32_t filter(__attribute__((unused)) void *type, SDL_Event *event) {
    return ((event->type == SDL_KEYDOWN) || (event->type == SDL_QUIT));
}

void viz_draw_labels(TTF_Font *font, SDL_Renderer *renderer) {
    for (size_t i = 0; i < LABEL_COUNT; ++i) {
        const int32_t label_size = snprintf(NULL, 0, "%zukHz", i);
        char label[label_size + 1];
        label[sprintf(label, "%zukHz", i) + 1] = 0;

        const SDL_Color WHITE = {MAX_COLOR_VALUE, MAX_COLOR_VALUE, MAX_COLOR_VALUE, MAX_COLOR_VALUE};
        SDL_Surface *surface = NULL;
        surface = TTF_RenderText_Blended(font, label, WHITE);
        CHECK(NULL == surface, SDL_GetError)

        SDL_Texture *texture = NULL;
        texture = SDL_CreateTextureFromSurface(renderer, surface);
        CHECK(NULL == texture, SDL_GetError)
        int32_t texture_width = 0;
        int32_t texture_height = 0;
        CHECK(SDL_QueryTexture(texture, NULL, NULL, &texture_width, &texture_height), SDL_GetError)

        CHECK(SDL_SetRenderDrawColor(renderer, 255, 255, 255, 255), SDL_GetError)
        CHECK(SDL_RenderDrawLine(renderer, i * WIDTH / 20, HEIGHT / 2, i * WIDTH / 20, HEIGHT / 2 + 5),
              SDL_GetError)

        SDL_Rect destination = {
                .x = ROUND((float) i * WIDTH / LABEL_COUNT),
                .y = ROUND((float) HEIGHT / 2 + LABEL_Y_SPACING),
                .w = texture_width,
                .h = texture_height,
        };
        CHECK(SDL_RenderCopy(renderer, texture, NULL, &destination), SDL_GetError)

        SDL_FreeSurface(surface);
        SDL_DestroyTexture(texture);
    }
}

VIZ viz_init(void) {
    SDL_Window *window = NULL;
    SDL_Renderer *renderer = NULL;
    TTF_Font *font = NULL;
    CHECK((TTF_Init() != 0), TTF_GetError)
    font = TTF_OpenFont(TTF_FONT_PATH, 12);
    CHECK(NULL == font, SDL_GetError)
    CHECK(SDL_Init(SDL_INIT_VIDEO | SDL_INIT_EVENTS), SDL_GetError)
    window = SDL_CreateWindow("PulseSpectralyzer", SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED, WIDTH, HEIGHT, 0);
    CHECK(NULL == window, SDL_GetError)
    renderer = SDL_CreateRenderer(window, -1, 0);
    CHECK(NULL == renderer, SDL_GetError)
    SDL_SetEventFilter(filter, NULL);
    CHECK(SDL_SetRenderDrawBlendMode(renderer, SDL_BLENDMODE_BLEND), SDL_GetError)
    return (VIZ){window, renderer, font};
}

// logarithmic scaling
float_t viz_scale(float_t value, float_t max) {
    return (logf(10.0f * value + 1.0f) / logf(10.0f * max + 1.0f));
}

void viz_draw(VIZ viz, float_t *const data, const int32_t size) {
    const float_t step = ((float_t) size / (float_t) WIDTH);
    for (int32_t i = 0; i < WIDTH; ++i) {
        float_t sum = 0;
        for (int32_t j = 0; j < ROUND(step); ++j) {
            sum += data[ROUND(i * step + j)];
        }
        data[i] = 1 - powf(SCALING_LOG_BASE, -(sum / step));
    }

    CHECK(SDL_SetRenderDrawColor(viz.renderer, 0, 0, 0, 255), SDL_GetError)
    CHECK(SDL_RenderClear(viz.renderer), SDL_GetError)

    viz_draw_labels(viz.font, viz.renderer);

    for (int32_t i = 0; i < WIDTH; ++i) {
        SDL_Color color = hsv_to_rgb(
                (HSV){fmodf(HUE_OFFSET - ((data[i] * HUE_MAX_VALUE) / 2.F), HUE_MAX_VALUE), 1, 1});
        CHECK(SDL_SetRenderDrawColor(viz.renderer, color.r, color.g, color.b, 255), SDL_GetError)
        CHECK(SDL_RenderDrawLineF(viz.renderer,
                                  (float) i,
                                  (float) HEIGHT / 2,
                                  (float) i,
                                  ((float) HEIGHT / 2) - (viz_scale(data[i], 1.0f) * ((float) (HEIGHT - 10) / 2))),
              SDL_GetError)
    }

    SDL_RenderPresent(viz.renderer);
}

void viz_quit(VIZ viz) {
    SDL_DestroyRenderer(viz.renderer);
    SDL_DestroyWindow(viz.window);
    TTF_CloseFont(viz.font);
    TTF_Quit();
    SDL_Quit();
}
