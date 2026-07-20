# Task 22M Package 5 Coverage Repair Amendment

## Authority And Finding

This amendment is read with the approved Task 22M specification at
`docs/superpowers/specs/2026-07-19-ksr-fdmtest-v4-task22m-elephant-foot-slice-ordering.md`
(SHA-256 `5433110c60aa4aa7e72f193fbdecde07d8ca3556704320aae3d39a148a02e2ff`)
and plan at
`docs/superpowers/plans/2026-07-19-ksr-fdmtest-v4-task22m-elephant-foot-slice-ordering.md`
(SHA-256 `b5dd487ebe277982e26365377173fa3ecafc5bd31d4c1c5c267835f77aecede8`).
The Package 4 layout, Package 5 fixture layout, and Package 5 orchestration
layout amendments remain authoritative.

Independent Package 5 review rejected the otherwise green implementation for
two P1 coverage gaps: the complete fixed-source synthetic M aggregate was not
encoded by Rust, and the real-3MF matrix did not carry compensation-layer, XY,
and region-cardinality variants through the typed project boundary. This
amendment repairs only those omissions. It does not authorize production,
Cargo, feature, adapter, workflow, geometry, Flow, or wire-format changes.

The fixed source remains OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`. The Ares baseline remains commit
`fcd2c5728f4c0529f28bfc43c636507d61e263d8`, tree
`19557e2e520e6b6d0e758740fd00f57397b6fd2a`.

## Manifest Delta

Add exactly two real Rust test leaves:

- `crates/ares-core/src/project_slice/tests/compensation/synthetic.rs`;
- `crates/ares-core/src/project_slice/tests/compensation/fixture/options.rs`.

The existing `project_slice/tests/compensation.rs` may add only
`mod synthetic;`. The existing fixture root may add only `mod options;`,
expose constants/helpers to its descendant where necessary, extract its four
common semantic entries into a constructor returning `KsrArchive`, and expose
one private object-count wrapper around its existing exact-EOF M parser to the
synthetic sibling. Parser records and types do not widen visibility. Existing
enabled, disabled, anti-map, parser, and KSR bytes/assertions must not change.
`tests/support.rs`, production files, and every other test file remain
unchanged.

## Cumulative Manifest Override

The approved exact 58-path frame is replaced by an exact 62-path frame:

- the original 49 paths;
- the Package 4 amendment specification, plan, and kernel-test leaf;
- the Package 5 fixture amendment specification, plan, and checkpoint leaf;
- the Package 5 orchestration amendment specification, plan, and preflight
  leaf; and
- this amendment specification, plan, synthetic leaf, and real-3MF Option
  leaf.

Every earlier final exact-49, exact-55, or exact-58 manifest/content-frame gate
means this exact 62-path frame. No other path is authorized.

## Complete Synthetic Aggregate

`synthetic.rs` constructs the approved 19 cases from bounded input geometry
and typed Options, calls the real `apply_project_compensation` for every case,
collects source IDs 0 through 18 in order, and calls `task22m_oracle::encode`
once for the complete vector. Different initial widths and the LargeBed case
require separate real apply calls before the one aggregate encode; concatenated
independent frames are forbidden.

The cases are: large rectangle; narrow neck with a hole; tiny no-op; disabled;
raft; two-layer ramp; layer-count clamp; zero layers; empty surface layer;
per-layer 0.2/0.3 heights; initial 0.6; initial-to-outer 0.38; outer-to-object
0.52; absolute-zero auto width; negative auto width; 125 percent with selector
2 and nozzles `[0.4, 0.6]`; selector-3 fallback; LargeBed scale; and the
left/nested/right two-pass-union discriminant.

The test copies only those fixed inputs. It must not copy output geometry,
read or embed ignored evidence, inspect Git or OrcaSlicer, or branch on a name
or hash. Its exact repeated result is:

- magic `ARES22M\0`, 19 objects, source IDs `0..18`, transform zero;
- retained counts `1,1,1,1,1,3,2,0,1,2,1,1,1,1,1,1,1,1,1`;
- 10,351 bytes;
- SHA-256 `c112246ff48b280eb803082749d74315e771d073b0407e45afde536e37fcf46d`;
- exact EOF when parsed by the existing independent M parser.

The frozen test name is
`task22m_synthetic_aggregate_is_exact_complete_and_repeatable`.

## Real 3MF Option Matrix

The existing absolute 0.5, disabled, 125-percent selector-one anti-map pair,
and KSR archives remain unchanged. `fixture/options.rs` adds:

1. a selector-two archive with 125 percent initial width and nozzle diameters
   `[0.4, 0.6]`, proving the selected f32 width is 0.75 mm and producing exact
   M output 1,274 bytes / SHA-256
   `dd9aa8d9aec514345b85806edd088f55f47d7e7fd5da032cb4e012e49c3c6cb5`;
2. a compensation-layers-two archive, producing exact M output 1,274 bytes /
   SHA-256
   `36b51849cbe3cc73e002ba63310af37d21d335a050026c6713e9dfe18e573db0`,
   with layer zero matching enabled output, layer one inset by the fixed 0.075
   mm ramp, and raw `lslices` retained on both layers;
3. separate nonzero `xy_hole_compensation` and
   `xy_contour_compensation` archives whose loaded typed values reach exact
   `UnsupportedProjectFeature` keys in that order; and
4. the existing real control/modifier archive pair from `region_fixture`,
   whose serialized region Option difference yields one versus two retained
   regions and whose two-region member reaches exact
   `UnsupportedProjectFeature("multi_region_layer_slices")`.

Every new archive freezes deterministic ZIP and ordered semantic-entry
identities before its M call. Valid M-only Option variants retain exact small L
input 746 bytes / SHA-256
`70c9c246700b068e1085a2c719243fd94839bb169c3a062b06b42fd640147b2a`.
Rejected variants freeze their released predecessor result and exact error;
they do not invent an M frame. Each archive is rebuilt twice, and pairwise
entry comparison proves only the intended serialized Option changed.

The fixed ignored option probe remains runtime-inaccessible. Its independently
approved layers-two extension is 243 lines / SHA-256
`890dae82d1cfd2ec8dae015acadd1632abe088b018bd7aecfe5f392241b6441a`;
two clean runs produced the same M identity above and text 1,545 bytes /
SHA-256 `2f6987245cf1a8edaf073173451125fdc2c259ddcbc6cb6ee601d82f06a1a37b`.
The fixed oracle startup self-check already owns the XY and multi-region
rejection contracts.

The frozen test names are:

- `task22m_real_3mf_width_nozzle_and_layers_are_exact`;
- `task22m_real_3mf_xy_and_region_count_reach_exact_stage_gates`.

## Budgets And Acceptance

- `tests/compensation.rs`: below 50 physical lines;
- `tests/compensation/synthetic.rs`: at most 390 physical lines;
- `tests/compensation/fixture.rs`: at most 390 physical lines;
- `tests/compensation/fixture/options.rs`: at most 390 physical lines;
- every other approved budget remains unchanged.

The current 78/78 result is insufficient evidence and is the review RED. These
three tests are retrospective coverage repairs against existing production;
the implementation must not fabricate a historical behavioral RED. Expected
GREEN is exactly 81 Task 22M tests. Then rerun Task 22L, strict core clippy,
core WASM, fmt, macro/unsafe/LOC, and diff gates. Any production edit, changed
existing fixture byte, extra path, or failure to match fixed evidence requires
a new amendment and dual approval.
