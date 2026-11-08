# AdaptiveCubic per-region octree spacing

## Upstream boundary

Source revision: `345efec64c7b95a05ce5cf96df0dde66a158835c`.

- `src/libslic3r/Fill/FillAdaptive.cpp::adaptive_fill_line_spacing:304–323` allocates one spacing slot per printing region. A region contributes only with positive sparse density, an octree pattern and a nonempty fill-surface collection on at least one layer. Its own width, density and multiline determine its spacing. Width resolves against the maximum nozzle diameter; automatic width uses the f32 Flow boundary, while config arithmetic uses f64 and the widened f32 one-third constant.
- `src/libslic3r/PrintObject.cpp::prepare_adaptive_infill_data:1169–1190` maps positive region spacings to tree sets. Equal spacing and pattern kind reuse the same set; inactive slots have no set.
- `src/libslic3r/Fill/FillAdaptive.hpp::RegionOctrees:62–73` stores the region-to-set mapping. `src/libslic3r/Fill/Fill.cpp:1386,1619` selects the tree through `surface_fill.region_id` in emission and sparse anchoring.

## Rust boundary

Within `crates/ares-core/src/project_slice/`:

- `fill_entities/adaptive/preparation.rs::{ObjectSpacing,object_spacings}` stores AdaptiveCubic spacing slots in printing-region order. Inactive slots contain zero. Each active slot uses `adaptive/line_spacing.rs::configured` with that region's multiline. This replaces the older object-average AdaptiveCubic shell.
- `fill_entities/generate.rs` and `prepare_infill/bridge_over_infill/sparse_anchoring/arms.rs` select the spacing by the existing fill's `region_id`. Neither consumer changes the production or ordering of grouped fills.
- `fill_entities/adaptive.rs::{AdaptiveObject,octree_for,append,anchoring_lines}` consumes the selected scalar. The existing object/spacing-bits/pattern-kind cache provides tree reuse without a new cache framework.
- Layer options and sparse anchoring borrow the object's spacing collection. Bridge candidate expansion borrows the same collection for the unchanged SupportCubic payload.

The data remains private and platform-neutral. No public API, filesystem, terminal or rendering dependency is introduced.

## Included and deferred behaviour

Included: AdaptiveCubic per-region width/density/multiline arithmetic, active-region admission, region-indexed selection in both consumers and reuse of equal-spacing AdaptiveCubic trees.

Deferred: SupportCubic per-region semantics and its original-surface payload; separated-body trees; mesh/belt transforms and global Z offsets; octree construction and filler geometry; region grouping, identity and ordering; other infill patterns and G-code formatting. The existing SupportCubic scalar remains separate and cannot substitute for an AdaptiveCubic slot.

## Test premise

The existing cases in `crates/ares-cli/tests/orca_parity/adaptive_octree_spacing.rs` exercise density and multiline differences between printing regions, a single-region baseline and a same-config two-volume baseline. The latter coalesces to one printing region and does not establish reuse between distinct printing regions.

For the density case, the two Native slots are approximately 9 and 2.25, rather than an object-wide 3.6. For the multiline case, they are approximately 9 and 27, rather than one object-wide 9. The assertions compare complete ordered Native/API/CLI output with generator-line-only normalization and require raw API/CLI equality. Fixture bytes and comparison logic remain unchanged.

## Failure modes

1. Averaging widths or densities loses the owning region's spacing.
2. Applying printing-region zero's multiline to another region changes its tree scale.
3. Admitting a region without fill surfaces creates a tree absent from the source mapping.
4. Selecting emission and anchoring spacing differently produces inconsistent sparse paths.
5. Mixing SupportCubic and AdaptiveCubic cache kinds reuses a tree with different payload semantics.
6. Altering grouping to obtain a convenient region ID changes an upstream input rather than implementing region-indexed selection.
