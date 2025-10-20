#ifndef SPADE_FFI_H
#define SPADE_FFI_H

#include <stdbool.h>
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

// C-compatible error structure
typedef struct {
    int32_t code;
    int32_t poly_id;
    size_t seg_idx;
} SpadeError;

enum {
    SPADE_ERROR_NONE = 0,
    SPADE_ERROR_CONSTRAINT_PANIC = 1
};

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
    const SpadePoint* const* hole_loops,
    const size_t* hole_loop_counts,
    size_t num_hole_loops,
    const SpadePoint* const* interior_loops,
    const size_t* interior_loop_counts,
    size_t num_interior_loops,
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

// Retrieve and clear the last error
bool spade_last_error(SpadeError* out_error);

#ifdef __cplusplus
}
#endif

#endif // SPADE_FFI_H
