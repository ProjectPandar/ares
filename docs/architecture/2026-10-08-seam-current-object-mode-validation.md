# Seam configuration and visibility belong to the current PrintObject

Slices A0 and A implement the
[parameter-scope spec](gcode-seam-parameter-scope-and-visibility-order.md),
original `4bea1d247`, A0 amendment `2076d208e`.
The original baseline includes the D fix (`dbcd52d5a`); main
`cdbfba4f9` changes web only, so evidence may be reused after rebase.
B and C remain separate slices. The double-distance precision
checkpoint is retained for C; this slice changes no distance formula.

## Source boundary and minimal port

Pinned upstream revision `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`OrcaSlicer/src/libslic3r/GCode.cpp:2533–2534` initializes export defaults;
the current-instance loop at `:5330–5332` then applies default region
configuration (`:5341`) and
`m_config.apply(instance_to_print.print_object.config(), true)` (`:5342`)
before setting the current layer (`:5343`). Perimeters invoke seam placement
at `:5771–5773`.

Rust `gcode_emit/motion/options/build/context.rs:39` selected the first
resolved object; `fields/assemble.rs:326–328` copied its seam mode into
shared MotionOptions. `motion/perimeter.rs:97–118` consequently skipped
Nearest selection on a later Nearest object when the first was AlignedBack.
The port initializes this one field from the true global default, then
refreshes it from `resolved.objects[object_index].object.seam_position`
immediately before `layers/object_entry.rs::emit` calls `motion::emit_layer`.
Tool selection replaces MotionOptions at `motion/tool_selection.rs:16`;
the enclosing `layers.rs:285–309` installs the tool before iterating its
object entries, so each object refresh follows that replacement. The
traditional timelapse callback does not replace MotionOptions.

The existing compatibility shell now carries this upstream object setting
at its current-object boundary. Candidate ordering, RNG, sampling, raycasts,
visibility arithmetic, comparisons, Gaussian and D's height formula stay
unchanged. No new API, allocation, fallback or private test probe is added.
Other first-object/region-derived options and support/skirt/brim behavior
are deferred; core remains platform-neutral.

An isolated uncommitted A visibility-scope trial without A0 failed the same witness
and left the complete Ares output SHA unchanged at
`9e14003642f98b6003b5d385ca606ddacb934b4a2b592b6cec12d2bd678c64ef`.
The trial was reverted; it is a
failed diagnostic, not GREEN. A0 was implemented
alone first. It fixed the initial difference but left a second-layer seam
RED, documented below; it did not restore complete equality.

The remaining A source discrepancy is
`GCode/SeamPlacer.cpp:1431–1441`: `SeamPlacer::init` reads each PrintObject's
seam preference and passes it to `compute_global_occlusion`.
`raycast_visibility`, `:165–167`, adds front_adjustment only for that
object's spAlignedBack. Rust `seam_placement.rs::apply` instead computed
`any(AlignedBack)` and supplied that flag to every object's visibility
model. The A edit removes this shared flag and derives it from the current
placement inside the object's existing visibility construction. It leaves
front-adjustment arithmetic untouched, deferring C's operation order.

Effective per-object settings are traced through existing source:

- `project/model_settings/object_metadata.rs:8–40` dispatches object metadata
  into ObjectOptionOverrides.
- `options/object_options.rs:32–63` overlays present object fields on the
  global base and resolves without altering seam_position.
- `project/effective_config/candidates.rs:29–38` stores each source object's
  resolved ObjectOptions.
- `seam_placement/runtime.rs:26–40` selects that object's seam mode.

The tests verify stored object-scoped metadata and global defaults, then
observe complete public byte output with controlled sibling inputs. They do
not access crate-private resolved settings or add an accessor for testing.
Wire values `aligned_back` and `back` are distinct
(`options/process_options/object_source/enums.rs:67–78`).

## Identified Native producer

The generic comparator manifests mark cached producer identity unknown;
they are not used alone as provenance. Native-cache-v2 receipts bind each
input request and output hash to the real producer. The9 receipts for RED,
A0-only and joint GREEN use producer digest
`ae933746dd2c805397c2a62d80973a94d0b190abeccfa907cec4760de7ca6caf`,
seed0.
Native binary SHA256 is
`59fb88a2cb441713c2f572cb06e30e4ab58c22a676cacca1812372b9575e73ff`;
resources SHA256 is
`286fab66becc4c87ebb72dc8e66ec9d515e7f440d530bacebeb1f51407bb8a69`.
Real Native output, not a floating-point construction, supplies every RED
and GREEN reference. The only output normalization is generated-by.

## Bounded reachability and fixtures

The unchanged source fixture is the existing
`tiq2-dumbbell-long-loop.3mf`, SHA256
`90288cb1f8904fc245a792be9cc8d30e5ea2762f6faff4b573d362fec231bc23`.
The candidate generator duplicates its model with distinct resource/object
IDs and UUIDs, updates relationships/build/plate associations, names the
objects `bias-driver.stl` and `nearest-target.stl`, and sets explicit
object-level seam metadata. Both first and regular layer heights are0.2.
No D/adaptive-height behavior is needed to reach A0.

The finite domain is four layouts (left/right, right/left, front/back,
back/front), target rotations0/90 degrees, and driver modes nearest or
aligned_back while the target stays nearest:16 two-object cases, plus
single-nearest and single-aligned_back controls. Offline checks validate
XML, mode scopes, build/plate counts and equal layer heights.

Fresh real seeded Native comparisons produced **10 PASS,8 DIVERGENT**.
All8 mixed-mode cases diverged; all8 two-nearest and both single-object
controls passed. The first changed object movement in every mixed case
belongs to `nearest-target.stl`. This movement-only diagnostic attribution
is not a separate comparator or a relaxed PASS criterion: the actual test
always compares the complete output, including M73 and object-id comments.

The diagnostic external-manifest test was replaced before RED commit; no
committed test depends on external fixture paths.

Three copied inputs are frozen in `tests/parity/validation/`:

| Fixture | Domain | SHA256 |
|---|---|---|
| `seam-mixed-object-bias.3mf` | driver(130,110) aligned_back; target(90,110),rot90 nearest | `3ab40b2ea7cf55598aecf52b0de61404c7b890737a2b8e638a47f6e2acd2c9f8` |
| `seam-two-nearest-control.3mf` | Same geometry/layout, both nearest | `9df9c0de9faf1ab6854c74b58d372f4f78e9a4d773987edd481b428ddf522115` |
| `seam-aligned-back-single-control.3mf` | Single aligned_back Dumbbell | `08f309f90e688f48d1cf0a012e45dff6ff484d40bca28cbb6adeff603c3285cc` |

## Independent whole-output RED

RED commit `96d1836ad1a8f473f9809548f3cb6375bbdcf7f2` precedes production edits.
Module: `ares-cli/tests/orca_parity/runner/seam_object_bias.rs`.
The three E2E cases pin fixture hashes, verify raw scoped modes and project
defaults, rerun real Native, and compare the public slicing API's complete
output, normalizing only the generated-by line.

Standalone fresh seeded RED:2 PASS/1 FAIL,1.736s,nextest100.
The mixed case is DIVERGENT; both sibling controls PASS.
RED executable SHA256
`a188cf86afdaf7b267dd0f078429c3e5f22014c3ea4a038e614a8e1fd084ff8a`.

Whole-output first difference is byte2322,line105,column1:
Native `G1 X120.31 Y106.31 E.22677`, Ares `M73 P60 R0`; Native emits that
M73 on line106 instead. Expected normalized length28343, actual28266.
Nothing was filtered to move this first difference later.

The nearest object's seam-entry travel is different at raw line120:
Native `G1 X90.418 Y107.695 Z.6`, Ares `G1 X93.69 Y119.69 Z.6`, followed
by Z.2. The preceding driver's geometric prefix and wipe path agree;
the changed nearest seam affects the complete travel/loop/wipe sequence
and timing, consistent with the earlier M73 displacement. The frozen
mixed test requires whole-file GREEN, including M73, not just the seam line.
The single-aligned_back control and every two-nearest control already pass,
which provides controls for this mixed-object configuration failure; it
does not by itself establish an independently reachable visibility defect.

## GREEN and required gates

### A0-only retained intermediate RED

A0 boundary commit `5b42d7b47ccd2595ecea492d7146508df2fc6ad0` is not a
whole-output GREEN. Fresh directed comparison gave2 PASS/1 DIVERGENT in
2.953s.
Both sibling controls PASS. The original M73/first-layer seam difference
disappeared, but the first difference moved to byte4604, raw line212,
column8: Native `G1 X89.599 Y112.057 Z.8` versus Ares
`G1 X89.6 Y107.957 Z.8`, followed by Z.4. Ares raw SHA256 is
`e0f5c8e45a72095ebbfd1162e8c9bd3bb51014a8ba7b0587822328ad3631a10f`;
the executable SHA256 is
`0193d376ef9139c856394e3643dc1e661642322f39f27fd5e84affccd09b5f6d`.

RED and A0-only Native outputs are byte-identical after removing only the
generated-by line, normalized SHA256
`c73fc2bafccbf853d6cfbb5760a7866113963d65d4e3db0405d1e21edeb7ed40`.
Raw Native SHA changes only with that allowed header; both complete files
are retained. The residual is therefore an Ares change,
not a new oracle output. A's per-object visibility edit is tested on top of
this retained A0-only boundary to establish whether it fixes the residual.

### Joint A0+A directed GREEN

Adding only A's per-object bias correction on A0 restores3/3 PASS in1.485s.
The mixed input's Ares raw
SHA256 is `ddbcb7a5dbf0333f77aca1b49e3fafee68009fad766dd833c57d5e37b52deb33`;
the executable SHA256 is
`efcb8ee320a52189f70a8d53f5fd206a36054e7fc80e314556d94a996ba9c2d4`.
The mixed Native normalized SHA is again identical to the full hash
recorded above. Both sibling Ares outputs are also byte-identical to RED.
This single-source-boundary addition removes the A0-only second-layer
residual, providing the attribution evidence for A. Neither
partial fix alone passes the witness; both source boundaries are needed.

| Producer state | Mixed fixture | Both sibling controls | First difference |
|---|---|---|---|
| RED | DIVERGENT | PASS | byte2322,raw105,M73 placement |
| A only, reverted diagnostic | DIVERGENT, Ares SHA unchanged | PASS | Same as RED |
| A0 only | DIVERGENT | PASS | byte4604,raw212,second-layer seam travel |
| A0+A | PASS | PASS | None, complete ordered bytes |

The two boundaries have separate commits. After a zero-conflict rebase onto
`cdbfba4f9`, RED is `e3b13c142`, amendment `609f61c61`, A0
`c8c7b3410`, and A's joint GREEN `972cfbca0`. The runtime/test/fixture diff
against the pre-rebase gated candidate `3ef1ada0b` is empty. Main's intervening changes
since D contain no crates/tests/scripts/Cargo/.config changes.

The independent base is RED96d1836ad.
Fresh same-round seeded controls completed: base662/663 PASS in179.894s,
candidate663/663 PASS in127.975s, no timeout. Only the mixed seam witness
changes FAIL→PASS; both siblings remain PASS.
Official controls completed: base653/663 PASS in148.880s,
candidate653/663 PASS in113.211s, no timeout. Both have10 raw failures,
with two status/reason changes listed below.
Workspace nextest passed8053/8053 with9 skipped in718.881s;
WASM, final strict clippy and fmt passed; the earlier fmt line-wrap failure
remains in its original log.

### All status changes and official oracle failures

| Case | Seeded base→candidate | Official base→candidate | Classification |
|---|---|---|---|
| `runner::seam_object_bias::aligned_back_object_does_not_bias_nearest_object` | FAIL→PASS | FAIL→PASS | A0+A complete-output fix |
| `bridge_over_infill_filament::bridge_over_infill_multi_filament_x2d` | PASS→PASS | PASS→FAIL | INCONCLUSIVE-oracle-UB |

All other661 cases have unchanged raw status and reason. Seeded candidate failures are a
strict subset of base. Official raw failure sets are **not** a subset:
the bridge X2D case is the sole candidate-only raw failure. It is not an
Ares regression or a PASS; it keeps its known
oracle-UB classification and was rechecked serially below.

Candidate bridge X2D first asserted difference is byte57060,line1096
(raw line1097): Native
`; printing object ksr_fdmtest_v4.drc id:13771850873779538142 copy 0`
versus Ares's id0 line. Its complete asserted Ares SHA256 is
`331827159281658d6baac34427836ee19fe89242c123fcb32ae3757ccd4ed79d`,
identical to both sides of the accepted D report. Evidence is parsed from
the actual candidate failure arrays.
The original base raw PASS is preserved; it does not establish an initialized
Native oracle. This is the previously classified uninitialized
`Print.hpp:582 m_id`, read at `:468–469`, not an invented Ares object ID.

`narrowed_x2d_whole_file::narrowed_x2d_whole_file` remains FAIL
on both sides. First asserted difference is byte57075,line1097 at the same
id0 object label; Native IDs are9223372037920129024→4565949087785943040.
The complete asserted Ares SHA256 is identical on both sides:
`ff469e046fbd6f49e361bed645f184e5daed08366a9ce02d0f48a0dc26574543`.
It remains
INCONCLUSIVE-oracle-UB, never promoted to PASS.

The other8 official failures are unchanged missing initialized-reference
environment (`NotPresent`), retained as FAIL:

- `option_coverage::bookkeeping_export::bookkeeping_matches_complete_configured_filament_output`
- `option_coverage::initialized_object_id::initialized_reference_repeats_complete_ksr_output`
- `option_coverage::initialized_short_line::initialized_short_line_matches_complete_internal_fill_output`
- `option_coverage::initialized_stitch_junction::initialized_stitch_junction_matches_complete_internal_fill`
- `option_coverage::initialized_transition_snap::initialized_transition_snap_matches_complete_internal_fill`
- `option_coverage::single_volume_offset::single_volume_offset_matches_complete_rotated_prism_output`
- `runner::initialized_loop_chain::initialized_loop_chain_matches_complete_output`
- `runner::initialized_overhang_intersections::initialized_overhang_intersections_match_complete_output`

### Serial bridge X2D recheck

One fresh base→candidate single-test pair ran serially with independent
fresh cache roots, forced real Native slicing and the same single-case
filter. Base PASS in33.520s; candidate DIVERGENT in37.150s, no timeout.
Candidate's first difference remains byte57060,line1096 at the id0 label.
Its Native ID changes from the full-round13771850873779538142 to
9223372037920129024, while its complete asserted Ares SHA remains the
identical `33182715…4ed79d` recorded above. This confirms Native variability
at the known uninitialized member; it does not promote candidate to PASS.
The original full-round results and recheck remain separate evidence.

## Workspace gates

Workspace command is `cargo nextest run --workspace --no-fail-fast`. Optional-reference tests
returning without Native environment in this ordinary workspace run do
not replace the separate identified fresh Native comparisons above.
The workspace result is8053 PASS,9 skipped,35 slow in718.881s.

Main subsequently advanced to `1a8c92f55` (CLI exit-code
families, retained -102 metadata and the related SliceError fields), which
leaves G-code output unchanged: seeded/official evidence
is reused, but strict clippy and this slice's directed tests must run
after rebase. The workspace run completed on the gated `cdbfba4f9` baseline
before rebase, preserving its compiling source. Rebase onto `1a8c92f55`
then completed with zero conflicts: final RED `b2852ea44`, spec `b91896cd3`,
A0 `3d585ba51`, A joint GREEN `826db5ef2`. This slice's production/test/fixture
diff against previously gated `972cfbca0` is empty. Post-rebase fresh Native
directed tests passed3/3 in1.772s, with all three complete Ares output hashes
identical to pre-rebase GREEN.
The new executable SHA256 is
`1934150cd79c111639f43682d1ec0bb237cada99ba96c26328bcf743c3a9c294`.
Post-rebase strict clippy passed in2m16s; WASM check
`cargo check -p ares-wasm --target wasm32-unknown-unknown` passed in37.43s.

Final fmt initially required only a line wrap of the new current-object
seam assignment. `cargo fmt --all` applied that formatting, amended into
joint GREEN `d9d346effc1b88ba6382308e82332b7719f63e8c`.
The entire runtime/test diff from the post-rebase directed candidate is
this single line wrap;
Native/workspace/WASM evidence is reused for this non-semantic change.
Final strict clippy passed in1m22s and fmt passed, with exit0.

| Final gate | Result |
|---|---|
| Fresh paired seeded | 662/663→663/663; sole mixed seam fix; no timeout |
| Fresh paired official | 653/663→653/663; mixed seam fix and X2D oracle-UB status change; no timeout |
| Workspace nextest | 8053 PASS,9 skipped |
| Directed Native after main1a8c92f55 rebase | 3/3 whole-output PASS; all Ares hashes unchanged |
| Strict workspace/all-target/all-feature clippy | PASS with `-D warnings`, repeated after formatting |
| WASM ares-wasm check | PASS |
| Final fmt and diff whitespace check | PASS |

No failed original evidence was
overwritten, no comparator/fixture/assertion was relaxed, and no private
probe, new fallback, UB emulation or unrelated code cleanup was introduced.
