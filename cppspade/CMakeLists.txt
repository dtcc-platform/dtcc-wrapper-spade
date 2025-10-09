cmake_minimum_required(VERSION 3.15)
project(SpadeCppWrapper VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Find the Rust library
set(SPADE_FFI_LIB_DIR "${CMAKE_CURRENT_SOURCE_DIR}/target/release")

# Determine the library name based on platform
if(APPLE)
    set(SPADE_FFI_LIB "${SPADE_FFI_LIB_DIR}/libspade_ffi.dylib")
    set(SPADE_FFI_STATIC "${SPADE_FFI_LIB_DIR}/libspade_ffi.a")
elseif(UNIX)
    set(SPADE_FFI_LIB "${SPADE_FFI_LIB_DIR}/libspade_ffi.so")
    set(SPADE_FFI_STATIC "${SPADE_FFI_LIB_DIR}/libspade_ffi.a")
else()
    set(SPADE_FFI_LIB "${SPADE_FFI_LIB_DIR}/spade_ffi.dll")
    set(SPADE_FFI_STATIC "${SPADE_FFI_LIB_DIR}/spade_ffi.lib")
endif()

# Check if the library exists
if(NOT EXISTS ${SPADE_FFI_LIB} AND NOT EXISTS ${SPADE_FFI_STATIC})
    message(FATAL_ERROR "Rust FFI library not found. Please run 'cargo build --release' first.")
endif()

# Create C++ wrapper library
add_library(spade_wrapper SHARED
    src/spade_wrapper.cpp
)

target_include_directories(spade_wrapper
    PUBLIC
        $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
        $<INSTALL_INTERFACE:include>
)

# Link against the Rust FFI library (prefer shared library)
if(EXISTS ${SPADE_FFI_LIB})
    target_link_libraries(spade_wrapper PRIVATE ${SPADE_FFI_LIB})
else()
    target_link_libraries(spade_wrapper PRIVATE ${SPADE_FFI_STATIC})
endif()

# Platform-specific linking
if(APPLE)
    target_link_libraries(spade_wrapper PRIVATE "-framework CoreFoundation" "-framework Security")
elseif(UNIX)
    target_link_libraries(spade_wrapper PRIVATE pthread dl m)
endif()

# Build examples
option(BUILD_EXAMPLES "Build example programs" ON)

if(BUILD_EXAMPLES)
    add_subdirectory(examples)
endif()

# Installation
install(TARGETS spade_wrapper
    LIBRARY DESTINATION lib
    ARCHIVE DESTINATION lib
    RUNTIME DESTINATION bin
)

install(DIRECTORY include/
    DESTINATION include
    FILES_MATCHING PATTERN "*.h"
)

# Copy Rust library to installation directory
if(EXISTS ${SPADE_FFI_LIB})
    install(FILES ${SPADE_FFI_LIB} DESTINATION lib)
endif()
