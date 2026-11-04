# ARD: epsilon-aware complex layer-range boundary

Status: independent follow-up to merged Stage2 `083cb1862`.

1. Own the rewrite at PrintObjectSlice.cpp::layer_range_first (244–255),
   ::layer_range_next (258–266), and their complex slices_to_regions caller
   (338–350, 395–398). Rust destination is project_slice/region_slices/complex.rs
   and source-specific lookup helpers, not a new range graph or pipeline.
2. First lookup and advancing lookup are distinct. Complex assignment invokes
   next even on the first layer after initializing first. EPSILON is upstream's
   double 1e-4 (libslic3r.h:52); actual Z originates as float. Preserve comparison
   order, exact-adjacency rule and inclusive advancement predicate.
3. Fast shuffle (PrintObjectSlice.cpp:290–312) uses exact range comparisons and
   must not receive the complex tolerance. Existing records_at_z is a temporary
   compatibility shell only for the unported complex lookup; its exact fast
   behavior is not a defect to globally replace with epsilon selection.
4. A new legal runtime-Native witness must force overlap/complex routing before
   asserting an epsilon discrepancy. The merged single-volume wall-loop case
   cannot prove this helper path. Native full streams are the independent oracle;
   diagnostic selections cannot substitute for public byte API comparisons.
5. MM/fuzzy painting, spiral ranges, adaptive/painted profiles, active ZAA and
   other unproved option families are deferred. No new default, unsupported
   guard assertion, crash imitation, golden Native output, or relaxed comparison.
6. All production behavior stays platform-neutral inside core. New helper
   module size/scope must stay narrow; use an advancing iterator, not cloned
   configs or repeated graph construction. A source-only spec precedes
   independent runtime RED and production GREEN.

See [epsilon lookup spec](../superpowers/specs/2026-10-08-layer-range-epsilon-lookup.md).
