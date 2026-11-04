# Slice R: layer-range configuration and volume slicing

Upstream revision: `8500fcdcc`.
This refines [the P/R boundary](object-layer-profile-and-range-slicing-spec.md)
and does not replace its remaining canonical gates.

## Dependency and owning boundary

R ports the remaining `PrintObject::slice_volumes` range admission/routing,
using `PrintApply`'s existing region graph. P must merge first:
`PrintObject.cpp:3805-3833` selects a stored profile before
`Slicing.cpp:232-304` creates one from ranges. At `Slicing.cpp:249`,
`it_range->second.option("layer_height")->getFloat()` dereferences the option
pointer directly, before checking interval width. There is no default-height
substitution at that expression. Valid stored profiles avoid this call;
missing, top-mismatched or first-height-mismatched profiles do not.

The current Rust range generator skips None heights. That scaffold is not
Native authority for the missing-option route. R must not use it to claim
acceptance or invent a normal-height default. The existing rejection remains
for this unresolved route. Native behavior and whether
an earlier stage actually inserts a height option will be investigated below.

## Source and Rust destination

All upstream paths are under `OrcaSlicer/src/libslic3r`; Rust paths below are
under `crates/ares-core/src`.

| Upstream owner | Included behavior | Rust owner |
|---|---|---|
| `Slicing.cpp:232-304`, `layer_height_profile_from_ranges`; :807-866, `generate_object_layers` | Explicit-height schedules, actual generated slice Z; P profile bypass for region-only ranges | Reuse `project_slice/profile.rs`, `layers.rs`, `parameters.rs`; no second sampling loop |
| `PrintObject.cpp:3805-3833`, `update_layer_height_profile` | Exact first-height and top-tolerance selection governs whether regeneration occurs | Reuse P selection in `profile.rs`; minimal `planning.rs`/`capabilities.rs` wiring |
| `PrintApply.cpp:342-395`, `LayerRanges::assign` | Continuous partition, earlier lexicographic range trims later overlaps, default gaps | Existing `project/effective_config/layers.rs` |
| `PrintApply.cpp:958-1027`, `generate_print_object_regions` | Config deduplication, per-volume occupancy and mappings | Existing `effective_config` and `volume_regions`; no new registry |
| `PrintObject.cpp:3664-3705`, `region_config_from_model_volume` | Model-part object → volume → material → range precedence; modifier starts with parent region and its own overrides | Existing `options/region_options/merge.rs`, effective candidates; preserve modifier ownership |
| `PrintObjectSlice.cpp:100-136`, ranged `slice_volume` | Filter float Z in half-open intervals, slice once, scatter into original aligned layer slots | New private `project_slice/raw_intersections/ranges.rs`; minimal parent wiring |
| `PrintObjectSlice.cpp:148-228`, `slice_volumes_inner` | Volume ID order, model/negative/modifier participation, single-range shortcut, active ranges per volume, empty result removal | Reuse existing raw intersections and source-volume identities |
| `PrintObjectSlice.cpp:244-266`, `layer_range_first/next` | Initial exact-boundary next-range choice and subsequent EPSILON cursor advancement | Existing `region_slices/range_lookup.rs`; verify rather than independently redesign |
| `PrintObjectSlice.cpp:1149-1199`, `PrintObject::slice_volumes` | Allocate all regions, route by volume/range, trim empty top layers | Existing `region_slices`, state and layer owners |

Current `raw_intersections.rs:271-292` rejects any missing range height or
region override other than wall_loops. `capabilities.rs:11-23` rejects mixed
Some/None heights before object parameters/profile selection. These are the
specific compatibility shells to narrow/retire after RED; unrelated centering,
shared-mesh, spiral and ZAA contracts are untouched.

Range admission must follow each object's actual selected-profile decision,
not file presence or the first object. Compute parameters through the existing
planner and reuse P's selection criteria before allowing a missing-height
range. If selection regenerates and a height is missing, retain the existing
rejection rather than reaching the Rust None-skip scaffold. Do not introduce a
new public API, second profile validator or additional rejection category.
The exact private wiring will follow the rebased planner's structure.

## Included and deferred

Included:

- All-explicit-height ranges with density, width and speed overrides supported
  by existing typed region consumers; these give independent RED witnesses.
- Valid retained stored profile with region-only or mixed height/region ranges;
  schedule comes from P and configuration from actual per-layer slice Z.
- Half-open per-volume occupancy filtering using f32 slice planes, preserved
  empty layer slots, volume order and original identities.
- Range-last model-part overrides and modifier parent/config semantics.
- Overlap, gap, epsilon and top-empty-layer controls under uniform/nonuniform
  schedules. Existing range cursor/partition modules stay the owners.

Deferred:

- Missing height when profile regeneration is required: retain existing
  rejection pending dedicated Native analysis.
- Spiral mode with multiple range regions (`PrintObjectSlice.cpp:213` assertion),
  ZAA, mesh-adaptive profile generation, painted multimaterial/tool mapping,
  new support algorithms and unrelated volume-transform/shared-mesh admission.
- P's independent no-profile precise-Z/shrinkage finding.

No new whitelist, Unsupported guard, public diagnostic API, regions[0],
legacy fallback or independent Ares pipeline is part of R. New private files
must be ≤400 LOC and keep core portable without filesystem/Native calls.

## Public Native-backed RED and GREEN

Use authored 3MF input with source provenance/input SHA only. Use existing
`OrcaRunner` slice/cache adapters and the public validated byte API. Official
and seeded compare every ordered output byte through the unchanged
`compare_ordered_bytes_generator_only`; assert exact layer Z/HEIGHT sequences
and boundary-specific observable path/E/F changes in addition, not instead.
Native products and gdb data are not committed.

| Witness | Contract |
|---|---|
| R0 | Existing no-range, height-only and wall_loops complete-output controls remain identical |
| R1 | Explicit heights + sparse density change; main range guard yields real public RED |
| R2 | Explicit heights + width and speed independently; Native accepts and E/F/path changes occur within the intended layers |
| R3 | Overlap/gap/epsilon inputs with nonuniform stored profile; full stream and actual boundary layer identity |
| R4 | Retained valid profile + mixed Some/None heights, then profile + all-region-only ranges; Native acceptance avoids regeneration |
| R5 | Multipart and modifier spanning a boundary; parent mapping, per-volume occupancy and full output |
| R6 | Range override conflicts with model-part volume override; Native range-last precedence |

Run runtime Native first. Native rejection, timeout, nondeterminism or unrelated
Ares differences are reported before changing input context or scope. A passing
control is not labeled RED; main must fail the new public comparison/admission.
R gets its own RED then GREEN after P, not folded into P.
Any expectation reversal or retirement of an existing assertion must state its
exact old/new text and source reason. No blanket retirement of guard tests is
implied by this spec. Other assertions remain.

## Missing-height diagnostic probe (not acceptance)

During R, capture one identical authored input with a missing height and no
valid retained profile, official and seeded each three times, sequential fresh
cache/artifact roots. Keep every outcome; no retries or cherry-picking.
Record exit/diagnostic, layer vector, whole-output SHA if emitted, producer
identity and original/exported project member bytes. This is observation, not
PASS or authority for an undefined path.

Trace whether the imported ModelConfig has a layer_height option when
`Slicing.cpp:249` executes; determine whether an earlier import/apply stage
substitutes one or the direct dereference sees null. If needed, use the existing
Native binaries via explicitly configured gdb (nix invocation for diagnostic
use where allowed); do not rebuild Native. Report observed determinism and
source path before any retirement decision. No production change to
this missing-height boundary precedes that decision.

## Gates

P lands before R. Directed runs use a new cache/artifact root every invocation.

R acceptance: official/seeded directed including existing profile/range E2Es;
default-env workspace; strict all-target/all-feature clippy; fmt; core/vgcode/
wasm wasm32 check; canonical candidate-only full.
If inventory/domain/sweep registration changes, include option_coverage entire
group (including non-Native tests) in both modes. Audit raw-output changes;
non-range controls stay byte-identical and every other changed stream is
reported.
