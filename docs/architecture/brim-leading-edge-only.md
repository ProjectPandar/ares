# `BrimType::btLeadingEdgeOnly` — Flat-Bed Parity and Belt Admission Boundary

## Upstream source (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`)

- `src/libslic3r/PrintConfig.hpp:455-467`: declares `BrimType::btLeadingEdgeOnly` (enum index `7`, appended last). Documents that on a non-belt printer it behaves as `btOuterOnly`.
- `src/libslic3r/PrintConfig.cpp:557-567, 2116-2140`: registers wire token `"leading_edge_only"` and label `"Leading edge only"` on `brim_type`.
- `src/libslic3r/Print.hpp:419-424` and `src/libslic3r/Print.cpp:852-854`: `PrintObject::has_brim()` and `Print::has_brim()` treat any `brim_type != btNoBrim` with `brim_width > 0` (and no raft) as having a brim.
- `src/libslic3r/Brim.cpp`:
  - `use_brim_efc_outline` (`:56-62`) and bottom outline swap (`:512-518`): applies EFC bottom-layer region polygons for `btLeadingEdgeOnly` identically to `btOuterOnly`.
  - `outer_inner_brim_area` (`:505-510, 581-616`): sets `has_outer_brim = true` and `has_inner_brim = false` for `btLeadingEdgeOnly` on flat-bed printers, emitting the outer ring and the inner hole exclusion mask.
  - Support no-brim mask (`:642-666`): adds first-layer normal (`stInnerNormal`) and tree (`stInnerTree`) support contours to `no_brim_area` for `btLeadingEdgeOnly`.
  - Contact retention (`:725-755`): retains only brim islands within `2 * brim_flow.scaled_spacing()` of `objectIslands` or another brim island.
- `src/libslic3r/PerimeterGenerator.cpp`:
  - `PerimeterGenerator::outer_walls_first_for_brim` (`:3204-3207`) and `brim_puts_outer_walls_first` (`:3209-3212`): returns `brim_width > 0 && (brim_type == btOuterOnly || brim_type == btOuterAndInner || brim_type == btLeadingEdgeOnly)`.
  - Consumed on layer 0 by Classic wall ordering (`:1858-1865`) and Arachne wall ordering (`:2909-2911`) to print outer walls before inner walls.
- `src/libslic3r/Support/SupportCommon.cpp:416-446`: `generate_raft_base` sets `brim_outer = true` and `brim_inner = false` for `btLeadingEdgeOnly` (`:420-422`), expanding first-layer contours by `brim_object_gap` before trimming first-layer support expansion.
- `src/libslic3r/Print.cpp:2202-2211`: `Print::validate` pushes a non-blocking advisory warning when `btLeadingEdgeOnly` is used on a non-belt printer (`OrcaSlicer.cpp:7043-7045` logs it at warning level without failing the slice or adding it to `result.json`).
- `src/libslic3r/Support/SupportSpotsGenerator.cpp:748-776`: adds outer-brim adhesion support bonus for `btLeadingEdgeOnly` (`:755-757`) during support-spot stability analysis.
- `src/libslic3r/BeltBrim.cpp:438-600` (specifically `:504-525`) and `src/libslic3r/PrintObject.cpp:1370-1393`: belt-printer brim generation (`make_belt_brim`, `belt_brim_clip_leading_edge`, per-layer belt brim bands, and apron prologue).
- `src/slic3r/GUI/Tab.cpp:3628-3638`: UI option visibility filter on non-belt printers.

## Rust destination

- Option declaration and metadata (already present):
  - `crates/ares-core/src/options/process_options/object_source/enums.rs` (`ProcessBrimType::LeadingEdgeOnly`)
  - `crates/ares-core/src/options/registry/definitions/metadata.rs` (`"leading_edge_only"`, `"Leading edge only"`, key `7`)
- Capability admission lift:
  - `crates/ares-core/src/project_slice/capabilities.rs`: remove the blanket `ProcessBrimType::LeadingEdgeOnly` rejections from `validate` and `validate_settings`, leaving all other capability guards intact.
  - `crates/ares-core/src/project_slice/belt.rs`: retain the belt-printer guard (`object.object.brim_type != ProcessBrimType::NoBrim` -> `SliceError::UnsupportedProjectFeature("belt_printer: brim")`) because belt brim generation (`BeltBrim.cpp`) is not yet ported.
- Flat-bed brim generation and masks:
  - `crates/ares-core/src/project_slice/gcode_emit/brim.rs`: admit `ProcessBrimType::LeadingEdgeOnly` in `prepare_area` type dispatch, `support_mask`, the outer-only area branch, and `retain_contact`.
  - `crates/ares-core/src/project_slice/gcode_emit/brim/outline.rs`: admit `ProcessBrimType::LeadingEdgeOnly` in `efc_enabled`.
- First-layer outer-wall-first ordering:
  - `crates/ares-core/src/project_slice/perimeters/classic/perimeter_append.rs`: include `ProcessBrimType::LeadingEdgeOnly` in `transform_object` (`wall_sequence`) and `classify_inactive`.
  - `crates/ares-core/src/project_slice/perimeters/arachne/config.rs`: include `ProcessBrimType::LeadingEdgeOnly` in `outer_walls_first_for_brim`.
- Support first-layer brim trimming:
  - `crates/ares-core/src/project_slice/support_material/adhesion.rs`: include `ProcessBrimType::LeadingEdgeOnly` in `brim_trimming` (`outer`).
- Shared `has_brim` predicates already cover `LeadingEdgeOnly` via `brim_type != ProcessBrimType::NoBrim && brim_width > 0.0` (`project/effective_config/usage.rs`, `project_slice/extruders.rs`, `print/validate/estimated_tower.rs`, `project_slice/gcode_emit/footprint/first_layer.rs`, `project_slice/support_material/project.rs`, `project_slice/support_material/project/tree_input.rs`, `plater/plate_list/{wipe_tower,exclusion_filaments}.rs`, and `arrangement/imex/input.rs`).

## Included and deferred behaviour

Included:
- Flat-bed admission of global and per-object `brim_type = leading_edge_only` with zero config substitution and exact config-block roundtrip (`; brim_type = leading_edge_only`).
- Flat-bed degradation to outer-only brim generation (`Brim.cpp:506-508`), including width quantization, hole exclusion masks, support no-brim masks (`stInnerNormal` and `stInnerTree`), EFC bottom-outline selection (`brim_use_efc_outline`), and clipped-contact island retention (`retain_contact`).
- Layer-0 outer-wall-first ordering when `brim_width > 0` for both Classic and Arachne wall generators (`PerimeterGenerator.cpp:3204-3212`), and normal `wall_sequence` preservation when `brim_width == 0`.
- First-layer support brim trimming (`SupportCommon.cpp:420-422`) with `brim_outer = true` and `brim_inner = false`.
- Fail-closed belt-printer rejection (`SliceError::UnsupportedProjectFeature("belt_printer: brim")`) with zero G-code output whenever `belt_printer = 1` and `brim_type = leading_edge_only`.

Deferred:
- Belt-printer brim generation (`src/libslic3r/BeltBrim.cpp`, `PrintObject::has_belt_brim`, belt apron prologue, and belt brim G-code emission). Belt brims of all types remain explicitly rejected in `belt::admit`.
- `Print::validate` non-blocking advisory warning channel (`Print.cpp:2206-2210`, logged only under `--debug >= 2` in upstream CLI without changing `result.json` or G-code).
- Support-spot stability analysis (`SupportSpotsGenerator.cpp:748-776`); `ares-core` currently ports only curled-perimeter estimation (`:102-196`).
- GUI option hiding (`src/slic3r/GUI/Tab.cpp:3628-3638`); `ares-core` stays UI-neutral.

## Failure modes

- Blanket rejection remaining in `capabilities::validate` or `capabilities::validate_settings` blocks flat-bed `brim_type = leading_edge_only` (either globally or via per-object `model_settings.config` override).
- Missing `ProcessBrimType::LeadingEdgeOnly` in `brim.rs::prepare_area` silently skips brim emission on flat beds.
- Missing `LeadingEdgeOnly` in `brim.rs` `support_mask` fails to subtract first-layer normal or tree support footprints from the outer brim.
- Missing `LeadingEdgeOnly` in `brim.rs` `retain_contact` keeps severed brim fragments that no longer contact the object after clipping against bed boundaries or exclusions.
- Missing `LeadingEdgeOnly` in `brim/outline.rs::efc_enabled` ignores elephant-foot-compensated bottom slices when `brim_use_efc_outline = 1`.
- Missing `LeadingEdgeOnly` in Classic `perimeter_append.rs` or Arachne `arachne/config.rs` leaves layer 0 in `InnerOuter` wall order when `brim_width > 0`, or erroneously flips layer 0 when `brim_width == 0`.
- Missing `LeadingEdgeOnly` in `support_material/adhesion.rs::brim_trimming` omits the outer `brim_object_gap` expansion when trimming first-layer support expansion.
- Lifting `capabilities.rs` without preserving `belt::admit` would emit flat-bed outer brims on a belt printer instead of failing closed while `BeltBrim.cpp` is unported.

## Verification

Verified end-to-end through the public slicing API (`ares_core::slice_project`, `ares_core::slice_project_plate_validated`) and CLI (`ares`) against pinned OrcaSlicer 2.5.0 using strict complete-stream byte comparison (`golden::compare_ordered_bytes_generator_only`) across:
1. Flat-bed Classic walls with holes (`two-holes.3mf`): positive brim emission, hole clearance, layer-0 outer-wall-first order, `; brim_type = leading_edge_only`, and whole-G-code byte match.
2. Flat-bed Arachne walls (`two-holes.3mf` with `wall_generator = arachne`): layer-0 outer-wall-first order and whole-G-code byte match.
3. Flat-bed EFC bottom outline (`two-holes.3mf` with `elefant_foot_compensation = 0.2`, `brim_use_efc_outline = 1`): whole-G-code byte match.
4. Flat-bed clipped-contact retention (`clipped-contact.3mf`): severed brim pruning and whole-G-code byte match.
5. Flat-bed support interaction (`outer-inner.3mf` under both tree support and normal auto support with positive `brim_object_gap`): support no-brim mask and first-layer support brim trimming whole-G-code byte match.
6. Flat-bed Bambu Lab X1C profile (`x1c-adaptive.3mf`), zero-width control (`brim_width = 0`), and per-object `model_settings.config` override.
7. Belt-printer fail-closed rejection (`belt_printer = 1`, `brim_type = leading_edge_only`): both API and CLI reject with `belt_printer: brim` and emit no G-code.
