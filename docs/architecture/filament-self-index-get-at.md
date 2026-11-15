# Filament self-index selection uses ConfigOptionVector::get_at

## Upstream boundary

OrcaSlicer 2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`src/libslic3r/PrintConfig.cpp:11954–11962`, remaps the filament ID list last
using `opt_ids->get_at(variant_index[f_index])`.
`src/libslic3r/Config.hpp:676–680` returns the requested value when present,
otherwise the first value. The caller excludes empty ID lists.

## Decision

The Rust destination is `options/project_variants.rs::remap_filament_self_index`.
It must preserve in-range IDs, reuse the first ID for an index beyond list length,
and leave an empty list untouched. This is upstream selection behavior, not an
input-validation rule or an Ares-specific recovery policy.

A selected-profile overlay may rebuild one ID per logical filament while retained
variant columns still contain multiple variants per filament. Switching a synthetic
dual-nozzle project to four-tool profiles exercises this state through the public
`slice_with_profiles` API. Tests must use authored geometry and public presets,
not private project data, and retain repeatable slicing artifacts.

## Scope

Only the final ID selection changes. Variant-index resolution, profile overlay
ownership, flush normalization, geometry, G-code generation, and browser worker
lifecycle are unchanged. Existing Ares entry scaffolding remains a temporary
compatibility shell around the cited `DynamicPrintConfig` operation; this repair
does not introduce an independent slicing design.
