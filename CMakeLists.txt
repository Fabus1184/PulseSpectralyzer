cmake_minimum_required(VERSION 3.0.0)
project(PulseVizCli VERSION 0.1.0)

set(CMAKE_C_STANDARD 17)

find_package(SDL2 REQUIRED)
find_package(SDL2_ttf REQUIRED)

get_cmake_property(_variableNames VARIABLES)
list (SORT _variableNames)
foreach (_variableName ${_variableNames})

    message(STATUS "${_variableName}=${${_variableName}}")

endforeach()

file(GLOB SOURCES *.c)
add_executable(main ${SOURCES})

target_link_libraries(main m)

include_directories(main ${SDL2_INCLUDE_DIRS})

target_link_libraries(main ${SDL2_LIBRARIES})
target_link_libraries(main SDL2_ttf)
target_link_libraries(main pulse pulse-simple)
target_link_libraries(main fftw3f)

add_compile_options(-Wall -Wextra -ggdb -O0)

install(DIRECTORY ${CMAKE_SOURCE_DIR}
        DESTINATION ${PROJECT_BINARY_DIR}
        FILES_MATCHING PATTERN "*.ttf" )