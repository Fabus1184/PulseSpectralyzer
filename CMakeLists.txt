cmake_minimum_required(VERSION 3.0.0)
project(PulseVizCli VERSION 0.1.0)

set(CMAKE_C_STANDARD 17)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

find_package(SDL2 REQUIRED)
find_package(SDL2_ttf REQUIRED)

file(GLOB SOURCES src/*.c)
add_executable(main ${SOURCES})

target_link_libraries(main m)

set(CMAKE_SOURCE_DIR ${CMAKE_SOURCE_DIR}/src)

include_directories(main ${SDL2_INCLUDE_DIRS})
target_include_directories(main PUBLIC ${CMAKE_SOURCE_DIR}/include)

target_link_libraries(main ${SDL2_LIBRARIES})
target_link_libraries(main SDL2_ttf)
target_link_libraries(main pulse pulse-simple)
target_link_libraries(main fftw3f)

add_compile_options(-Wall -Wextra -Werror -ggdb -O0 -pthread)
