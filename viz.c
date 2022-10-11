#include "viz.h"

#define SDL_CHECK(f, e) if(f) { \
    fprintf(stderr, "Error in %s, line %d: %s\n", __func__, __LINE__, e()); \
    exit(1);                                 \
    }0

SDL_Window *WINDOW;
SDL_Renderer *RENDERER;
TTF_Font *FONT;

SDL_Color hsv_to_rgb(float h, float s, float v) {
#define f(n) (v - (v * s * fmax(fmin(fmin(fmod(n + h / 60, 6), 4 - fmod(n + h / 60, 6)), 1), 0)))
    return (SDL_Color) {f(5) * 255.f, f(3) * 255.f, f(1) * 255.f, 255};
}

int viz_loop() {
    SDL_Event e;
    if (SDL_PollEvent(&e) && ((e.type == SDL_KEYDOWN && e.key.keysym.sym == SDLK_q)
                              || e.type == SDL_QUIT)) {
        return 1;
    } else {
        return 0;
    }
}

int filter(void *_, SDL_Event *event) {
    if (event->type == SDL_KEYDOWN || event->type == SDL_QUIT) {
        return 1;
    } else {
        return 0;
    }
}

void viz_init() {
    SDL_CHECK((TTF_Init() != 0), TTF_GetError);
    SDL_CHECK(NULL == (FONT = TTF_OpenFont("Font.ttf", 12)), SDL_GetError);
    SDL_CHECK(SDL_Init(SDL_INIT_VIDEO | SDL_INIT_EVENTS), SDL_GetError);
    SDL_CHECK(NULL ==
              (WINDOW = SDL_CreateWindow("Title", SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED, 1500, 600, 0)),
              SDL_GetError);
    SDL_CHECK(NULL == (RENDERER = SDL_CreateRenderer(WINDOW, -1, 0)), SDL_GetError);
    SDL_SetEventFilter(filter, NULL);
}

void viz_quit() {
    SDL_DestroyRenderer(RENDERER);
    SDL_DestroyWindow(WINDOW);
    TTF_CloseFont(FONT);
    TTF_Quit();
    SDL_Quit();
}

void viz_draw(const float *data, int size) {
    SDL_CHECK(SDL_SetRenderDrawColor(RENDERER, 0, 0, 0, 255), SDL_GetError);
    SDL_CHECK(SDL_RenderClear(RENDERER), SDL_GetError);

    int width, height;
    SDL_GetWindowSize(WINDOW, &width, &height);
    SDL_CHECK(SDL_SetRenderDrawColor(RENDERER, 0, 255, 0, 255), SDL_GetError);

    for (int i = 0; i < size - size / width; i += size / width) {
        SDL_Color color = hsv_to_rgb(fmodf(90.f + (data[i] * -180.f), 360.f), 1, 1.f);
        SDL_CHECK(SDL_SetRenderDrawColor(RENDERER, color.r, color.g, color.b, 255), SDL_GetError);

        int x = i * width / size;
        float y = 0;
        for (size_t j = 0; j < size / width; j++) {
            y += data[i + j];
        }
        y /= (float) size / (float) width;
        SDL_Rect rect = {
                .x = x,
                .y = height / 2,
                .w = 1,
                .h = (int) -(10 + y * ((float) (height - 10) / 2.f)),
        };
        SDL_CHECK(SDL_RenderFillRect(RENDERER, &rect), SDL_GetError);
    }

    for (int i = 0; i < size; i += 1000) {
        char label[50];
        sprintf(label, "%.0fkHz", (float) i * 20.f / (float) size);
        SDL_Color color = {255, 255, 255, 255};
        SDL_Surface *surface;
        SDL_CHECK(NULL == (surface = TTF_RenderText_Blended((TTF_Font *) FONT, label, color)), SDL_GetError);
        SDL_Texture *texture;
        SDL_CHECK(NULL == (texture = SDL_CreateTextureFromSurface(RENDERER, surface)), SDL_GetError);
        int texture_width, texture_height;
        SDL_CHECK(SDL_QueryTexture(texture, NULL, NULL, &texture_width, &texture_height), SDL_GetError);

        SDL_CHECK(SDL_SetRenderDrawColor(RENDERER, 255, 255, 255, 255), SDL_GetError);
        SDL_CHECK(SDL_RenderDrawLine(RENDERER, i * width / size, height / 2, i * width / size, height / 2 + 5),
                  SDL_GetError);

        SDL_Rect destination = {
                .x = i * width / size,
                .y = (height / 2) + 10,
                .w = texture_width,
                .h = texture_height,
        };
        SDL_CHECK(SDL_RenderCopy(RENDERER, texture, NULL, &destination), SDL_GetError);

        SDL_FreeSurface(surface);
        SDL_DestroyTexture(texture);
    }
    SDL_RenderPresent(RENDERER);
}
