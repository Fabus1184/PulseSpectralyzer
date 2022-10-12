#ifndef TOOLS_H
#define TOOLS_H

#include <stdio.h>

#define CHECK(f, e, ...) if(f) { \
    fprintf(stderr, "Error in %s, line %d: %s\n", __func__, __LINE__, e == NULL ? "" : e(__VA_ARGS__)); \
    }

#define ROUND(x) (int) roundf(x)

#endif
