cmake_minimum_required(VERSION 3.16)
project(hyprlog_cpp_example LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# =============================================================================
# FFI Connection Explanation
# =============================================================================
#
# The connection between C++ and Rust works like this:
#
#   +------------------+     +------------------+     +------------------+
#   |   main.cpp       |     |   hyprlog.h      |     |   libhyprlog.so  |
#   |   (your code)    |---->|   (interface)    |---->|   (Rust code)    |
#   +------------------+     +------------------+     +------------------+
#         |                        |                        |
#         |  #include "hyprlog.h"  |                        |
#         |  (compile time)        |                        |
#         |                        |                        |
#         +------------------------+                        |
#                    |                                      |
#                    |  -lhyprlog (link time)               |
#                    +--------------------------------------+
#                                   |
#                                   |  LD_LIBRARY_PATH (runtime)
#                                   v
#                           Function calls work!
#
# =============================================================================

# ------------------------------------------------------------------------------
# Step 1: Find the Rust-generated library
# ------------------------------------------------------------------------------
# After running `cargo build --release --features ffi`, the library is at:
#   target/release/libhyprlog.so (Linux)
#   target/release/libhyprlog.dylib (macOS)
#   target/release/hyprlog.dll (Windows)

set(HYPRLOG_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../..")
set(HYPRLOG_INCLUDE_DIR "${HYPRLOG_ROOT}/include")
set(HYPRLOG_LIB_DIR "${HYPRLOG_ROOT}/target/release")

# Check if library exists
if(NOT EXISTS "${HYPRLOG_LIB_DIR}/libhyprlog.so")
    message(WARNING "
    =======================================================
    libhyprlog.so not found!

    Build it first:
      cd ${HYPRLOG_ROOT}
      cargo build --release --features ffi
    =======================================================
    ")
endif()

# ------------------------------------------------------------------------------
# Step 2: Create an imported library target
# ------------------------------------------------------------------------------
# This tells CMake about the pre-built Rust library

add_library(hyprlog SHARED IMPORTED)
set_target_properties(hyprlog PROPERTIES
    # Where the actual compiled code lives
    IMPORTED_LOCATION "${HYPRLOG_LIB_DIR}/libhyprlog.so"

    # Where the header file is (for #include)
    INTERFACE_INCLUDE_DIRECTORIES "${HYPRLOG_INCLUDE_DIR}"
)

# ------------------------------------------------------------------------------
# Step 3: Build the example executable
# ------------------------------------------------------------------------------

add_executable(hyprlog_example main.cpp)

# Link against the Rust library
# This does two things:
#   1. Adds include path for hyprlog.h (compile time)
#   2. Links libhyprlog.so (link time)
target_link_libraries(hyprlog_example PRIVATE hyprlog)

# ------------------------------------------------------------------------------
# Step 4: Set up runtime library path
# ------------------------------------------------------------------------------
# The executable needs to find libhyprlog.so at runtime.
# This adds the library directory to the executable's RPATH.

set_target_properties(hyprlog_example PROPERTIES
    # Embed library path in executable (Linux/macOS)
    BUILD_RPATH "${HYPRLOG_LIB_DIR}"
    INSTALL_RPATH "${HYPRLOG_LIB_DIR}"
)

# ------------------------------------------------------------------------------
# Build Instructions
# ------------------------------------------------------------------------------
message(STATUS "")
message(STATUS "=== Build Instructions ===")
message(STATUS "")
message(STATUS "1. Build the Rust library first:")
message(STATUS "   cd ${HYPRLOG_ROOT}")
message(STATUS "   cargo build --release --features ffi")
message(STATUS "")
message(STATUS "2. Build this C++ example:")
message(STATUS "   mkdir -p build && cd build")
message(STATUS "   cmake ..")
message(STATUS "   make")
message(STATUS "")
message(STATUS "3. Run the example:")
message(STATUS "   ./hyprlog_example")
message(STATUS "")
message(STATUS "===========================")
message(STATUS "")
