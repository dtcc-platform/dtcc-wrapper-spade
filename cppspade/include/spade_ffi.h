#ifndef SPADE_FFI_H
#define SPADE_FFI_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// C-compatible point structure
typedef struct {
    double x;
    double y;
    double z;
} SpadePoint;

// C-compatible triangle structure
typedef struct {
    size_t v0;
    size_t v1;
    size_t v2;
} SpadeTriangle;

// C-compatible edge structure
typedef struct {
    size_t v0;
    size_t v1;
} SpadeEdge;

// Opaque handle to triangulation result
typedef struct SpadeResult SpadeResult;

// Quality enum (matches Rust side)
typedef enum {
    SPADE_QUALITY_DEFAULT = 0,
    SPADE_QUALITY_MODERATE = 1
} SpadeQuality;

// Perform triangulation
// Returns opaque handle to result, or NULL on failure
SpadeResult* spade_triangulate(
    const SpadePoint* outer_points,
    size_t outer_count,
    const SpadePoint* const* inner_loops,
    const size_t* inner_loop_counts,
    size_t num_inner_loops,
    double maxh,
    SpadeQuality quality,
    int enforce_constraints
);

// Get number of points in result
size_t spade_result_num_points(const SpadeResult* result);

// Get number of triangles in result
size_t spade_result_num_triangles(const SpadeResult* result);

// Get number of edges in result
size_t spade_result_num_edges(const SpadeResult* result);

// Get points from result (copies into user-provided buffer)
void spade_result_get_points(const SpadeResult* result, SpadePoint* buffer);

// Get triangles from result (copies into user-provided buffer)
void spade_result_get_triangles(const SpadeResult* result, SpadeTriangle* buffer);

// Get edges from result (copies into user-provided buffer)
void spade_result_get_edges(const SpadeResult* result, SpadeEdge* buffer);

// Free the result
void spade_result_free(SpadeResult* result);

#ifdef __cplusplus
}
#endif

#endif // SPADE_FFI_H
