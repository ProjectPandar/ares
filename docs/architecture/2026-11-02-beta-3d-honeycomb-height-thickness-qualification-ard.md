# Beta 3D-honeycomb physical-height and thickness qualification

Status: proposed. No new witness has been executed. Candidate settings below are not qualified REDs or PASS results.

## Authority and prerequisite

Authority is OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`. This proposal starts at `8612802fec12baeaa32600edb8e0f3795d3eeb95`, which contains the accepted 3D-honeycomb rewrite and its public geometry witnesses.

Qualify the integrated implementation without rewriting its accepted history. Do not restore the older generator to manufacture a fresh RED for behavior already corrected by that rewrite.

This is qualification of the `libslic3r` rewrite boundary below. It introduces neither a fill algorithm nor an Ares-owned pipeline. No production change is proposed. A new mismatch requires source localization and a separate RED/fix boundary.

## Source contract

Upstream paths are under `src/libslic3r/`. Rust destinations are under `crates/ares-core/src/` after the prerequisite is integrated.

| Upstream owner | Rust destination | Contract under qualification |
| --- | --- | --- |
| `LayerRegion.cpp:965–997`, `prepare_fill_surfaces` | `project_slice/prepare_infill/fill_surfaces.rs` | With zero bottom shells, bottom surfaces become internal. This is the candidate route to first-layer sparse infill. No classification rewrite is proposed. |
| `Print.cpp:2580–2592,2662–2697`; `Flow.cpp:134–151,207–212` | Native oracle admission and `project_slice/perimeters/flow/{fill,spacing}.rs` | Native validates explicit sparse width against nominal height. Rounded-rectangle spacing uses f32 width and nominal height. Admission must succeed on both sides. No validation rewrite is proposed. |
| `Fill/Fill.cpp:1070–1095,1432` | `project_slice/group_fills/params/projection.rs`; `project_slice/fill_entities/three_d_honeycomb.rs` | Sparse pattern spacing uses nominal object height. `FillParams::layer_height` uses the actual physical layer height, not extrusion thickness. |
| `PrintObject.cpp:4962–5082`, `combine_infill`; `Surface.hpp:45–54` | Existing `project_slice/prepare_infill/combine_infill/process.rs` producer and surface metadata | Combined surfaces retain summed thickness and a distinct thickness-layer count. First-layer skipping, height cap and actual combined regions must be visible in evidence. No general combination rewrite is proposed. |
| `Fill/Fill3DHoneycomb.cpp:352–398`, `_fill_surface_single` | `fill/three_d_honeycomb.rs`; field-only normal-fill handoff | Density and physical height select the module branch. The previous-Z argument spans physical height times multiline count times `layersPerSlice`. |
| `Fill/Fill3DHoneycomb.cpp:296–324`, `makeGrid` | `fill/three_d_honeycomb/grid.rs` and `tops.rs` | Complete-top transition detection consumes both Z arguments and the thickness-layer count. |

The public witness destination is `crates/ares-cli/tests/orca_parity/runner/plater_stl_session/honeycomb_geometry.rs`. Reuse its session setup, raw artifacts and comparator. No generic comparator framework is proposed.

Use three separate quantities: physical layer height `h`, combined surface thickness `H`, and thickness-layer count `n`. On a uniform combined group, `H = n*h`, but `h` must still drive `layersPerModule` and `startOffset`. The extrusion flow may use `H`. Combining layers must not silently replace `h` with `H` in the generator.

## Why a separate small-module witness is needed

A density above 42% exercises the first arm of `density > 0.42 || layersPerModule < 2`. It does not independently qualify the second arm.

For ordinary uniform sparse layers with nominal height equal to physical height and validated width at least that height, `spacing/h >= pi/4`. At density at most 0.42 and multiline at least one, the pre-floor module expression is bounded below by `(1 + 1/sqrt(2)) * (pi/4) / 0.42`, approximately 3.19. The stored integer module count is therefore at least three. This argument is limited to those uniform-height assumptions.

An invalid width below nominal height is not an acceptable way to force the guard. The candidate instead changes the first physical height while preserving nominal-height spacing. Zero bottom shells make first-layer sparse infill source-reachable. No adaptive-layer feature is needed for this candidate.

## Bounded candidate matrix

Reuse the existing cube, profile catalog, public plater APIs and geometry harness. Keep the 0.4 mm nozzle, nonzero sparse density and explicit multiline settings. Keep smoothing disabled so this slice does not enlarge its dependency boundary. Freeze exact settings and input hashes only after successful Native/API admission.

| Candidate | Settings that distinguish it | Required branch evidence |
| --- | --- | --- |
| Small-module control | Nominal height 0.08 mm, first height 0.08 mm, sparse width 0.10 mm, density 40%, multiline 1, bottom shell layers/thickness zero, first-layer width 0.50 mm | First-layer internal infill exists. Actual entry scalars give the ordinary pre-fallback module count at least two. |
| Small-module guard | Same settings, first height 0.24 mm | Density arm remains false. Actual first-layer height and nominal sparse spacing give the pre-fallback module count below two and the source fallback zScale. |
| Two-layer combination | Height 0.20 mm, combination on, maximum combined height 0.40 mm, density 20%, multiline 1, complete-top false | An actual sparse combined surface has `n=2` and `H=0.40`, while physical `h=0.20`. |
| Two-layer cover | Same combination, complete-top true | A current/previous orientation transition exercises the `n=2` span in top completion. |
| Three-layer cover | Height 0.10 mm, combination on, maximum combined height 0.30 mm, density 20%, multiline 1, complete-top true | An actual sparse combined surface has `n=3`, `H=0.30`, `h=0.10`, and a cover transition. |
| Combined multiline cover | Two-layer cover settings, multiline 3 | A cover transition consumes the multiline-times-thickness span rather than either count alone. |

Using source f32 flow arithmetic, the first-layer pair predicts nominal spacing about 0.08283186 mm and pre-fallback module counts four and one. These are planning calculations, not observed branch evidence.

The existing ordinary centered witness supplies the uncombined control. Reuse the existing seven witnesses rather than copying them. Candidate cap values predict grouping under `combine_infill`'s exact `>= cap + EPSILON` comparison. A setting being enabled is not evidence that grouping or a cover transition actually occurred.

## Qualification sequence and evidence

1. Run Native-first admission for these six candidates. Retain effective settings, the positioned input, successful official command status and complete output. Native failure, an empty target role or public API rejection disqualifies that candidate. Do not relabel it as a geometry RED.
2. Obtain valid typed Native entry observations for physical height, spacing, density, multiline and thickness count. Bind any diagnostic Native build to the official stream by generator-only equality before using its internal observations. Capture a cover-transition witness for each cover case. An optimized-out value or guessed layout cannot qualify a branch.
3. Reconcile those observations with `Surface::thickness`, `thickness_layers`, both `makeGrid` Z arguments and the corresponding Rust handoff. Preserve scalar types and source arithmetic. Do not add committed instrumentation or a public geometry API.
4. Add the admitted cases through the existing public plater geometry harness. Extend its test artifacts only as needed to retain the case-specific height/count/transition evidence. Keep the existing ordered lexical motion/mode oracle unchanged, including extrusion amounts. No coordinate normalization, numerical tolerance, sorting or selected-layer-only comparison is allowed.
5. Compare on the integrated prerequisite. If the cases already pass, close this as tests-and-documentation qualification. Do not manufacture RED/GREEN history or add a speculative product fix. A genuine new mismatch stops at its first source-owned operation for a separate slice.

## Exit criteria

- All six admitted public witnesses match complete ordered geometry, with active branch evidence. Report selected, filtered, rejected and unqualified cases separately. Do not claim six qualified cases if any activation proof is missing.
- The original seven geometry witnesses stay passing. Freeze the relevant honeycomb/fill inventory on the execution revision and require zero PASS-to-FAIL transitions. Preserve known Native ironing crashes as baseline classifications.
- Rerun the original strict case on that execution revision. Report first strict and first non-M73 differences, including feedrates, without changing its assertion. Geometry qualification is not full-stream parity.
- Retain complete streams, input/settings hashes, typed observations, command/status records and repeatable verification scripts outside the repository.
- Use nextest for Rust tests, formatting and strict workspace/all-target Clippy. Build `ares-core` and `ares-wasm` for `wasm32-unknown-unknown`. Changed files remain at most 400 lines without new suppressions.

## Deferred boundaries

General adaptive/variable-height profile support, combination scheduling or clearance changes, anchoring and bridge algorithms, other fill patterns, smoothing extensions, Clipper2, timing, cooling and configuration-footer fixes are deferred. The older generator is replaced only by the accepted upstream port, never preserved behind a selector. This qualification does not authorize repairs in adjacent producers merely because their effects appear in a geometry comparison.
