# Absolute-E preamble reset comment: upstream rewrite slice

## Source boundary and destination

Port the comment serialization of `GCodeWriter::reset_e(true)` called by
`GCodeWriter::preamble`, not a new Ares reset pipeline. Upstream is pinned at
`8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `GCodeWriter.cpp:82–106`: preamble selects M82/M83 and calls reset_e(true).
- `GCodeWriter.cpp:480–503`: absolute-E reset emits `G92 E0`, appends exactly
  ` ; reset extrusion distance` when full_gcode_comment is true, then newline.
  Relative E emits no reset. The earlier flavor and writer-state checks remain
  outside this bounded serialization slice.
- `GCode.cpp:2038`: full_gcode_comment comes from print.config().gcode_comments.
- `Tab.cpp:2753,4546,5470–5471`: GUI verbose output and non-BBL relative-E option.

Rust destination is only
`crates/ares-core/src/project_slice/gcode_emit/machine.rs::append_flavor_preamble`.
Its bare absolute-E `G92 E0` is a temporary compatibility shell around the cited
upstream preamble/reset_e serialization. Replace only its missing conditional
comment; retain flavor admission, M82/M83 bytes, order and reset count.

## Evidence intake and authority

COORD assigned this W1-owned discrepancy after merging both prior W1 stacks;
base is current main `1a57f677f1ddbbf32ef14c319019da844a212d34`.
W3 evidence is `${EVIDENCE_ROOT}/w3-pressure-discovery-bdb47cefe/`:
README.md and verification.json record the admitted absolute-E verbose control
at older base bdb47cefe, Native twice stable, validated API and CLI successful,
raw API==CLI, whole difference byte1177,line48: annotated Native versus bare
Ares reset. W3 correctly excluded it from PressureEqualizer ownership and did
not patch it. This is inherited evidence, not fresh exact-main qualification;
the new public RED must reproduce it on the new base before production changes.

Use unchanged W3 stock archives, no new geometry, pressure-rate changes or
compatibility bypass. All controls use slope0; no PressureEqualizer port is
included. Absolute-E inputs already remove the incompatible before-layer G92
through the GUI-editable template; preserve that admitted input, and preserve
W3's rejected earlier attempt rather than treating it as parity RED.

| New frozen label | W3 source | SHA256 |
|---|---|---|
| absolute-verbose | absolute-e-admitted/inputs/control.3mf | 9911f8075c51e6e9481132c2c4c5457ec494addf984740811a1c8896c9c3219f |
| absolute-quiet | absolute-e-no-comments/inputs/control.3mf | 908b9d9ebb69b852bcba5d3c52d364b245350fdab31d067c0cdf69082cefbfde |
| relative-verbose | slope-comments/inputs/control.3mf | ef1951bd9f85d84ebfaf569b5a0ac4cf729e22e3c0c0d788ea5ad71b54a6e632 |

GUI reachability is source-constructive, not a claim of live GUI saves. The
relative control retains its stock relative-mode startup/before-layer templates;
it is a mode control, not an archive differing by exactly one setting.

## Public RED and GREEN contract

Call existing native_stream_case::check / qualify directly with the three
unchanged W3 archives. The Native validator closure checks only the upstream
precondition: absolute-verbose contains `G92 E0 ; reset extrusion distance`;
absolute-quiet and relative-verbose do not. Qualify reference and replay in both
modes globally before public execution. Whole ordered Native/API/CLI comparison
remains the existing oracle, not this precondition check.

Do not use or extend paint_color activity or used_extruder_case: they witness
tool selection and are the wrong seam for a reset-comment slice. The failed
cce75f02f predecessor, original SPEC and binaries remain preserved in STOP
evidence and a retained branch. COORD authorized this tests-only seam correction,
not a new scanner/helper or changes to activity, inputs, runners or comparators.
Register plainly inside existing case_support; root is already400LOC and must
not grow. No private-helper substitute.

On CPU18, jobs/test/parity controls1, both official and seeded: fresh independent
Native reference/replay and comment precondition checks for all three cases
finish globally before public execution. Require actual validated API and real
CLI success plus raw API==CLI. Compare entire ordered output with only the
existing generator-line allowance. RED: verbose absolute target genuinely fails
whole comparison; both controls pass. GREEN: all three match Native in both
modes. Never strip/reset-normalize comments to obtain equality.

Failure inventory, before writing tests/production:

1. Rejected or failed Native/API/CLI is infrastructure/input failure, not RED.
2. Absolute verbose missing suffix is the intended successful-output RED.
3. Adding suffix unconditionally breaks absolute quiet; changing relative/reset
   emission breaks the relative verbose control.
4. Wrong whitespace/newline/order, duplicated reset or edits to user templates
   must fail the whole ordered oracle, not be hidden by semantic assertions.
5. Unstable replay, changed input hashes, wrong Native comment precondition or
   raw API/CLI inequality stops qualification.
6. Root/module over400LOC, trailer omission or owner-adjacent edits stop review.

Commit SPEC first, tests-only RED second; record both-mode RED artifacts before
the smallest production GREEN. Retain evidence and binary copies at each stage.
Run fmt, strict workspace/all-target Clippy, WASM compile and directed cases;
nextest list must change only by the new plain registrations. Send REVIEW-REQUEST
and hold. No FULL/rebase/merge/push; integration window is scheduled separately.

## Included and deferred upstream behavior

Included: the preamble absolute-E reset's conditional comment serialization.
Deferred and unchanged: reset_e's other callers, force/zero-E writer cache,
toolchange/unretract/object-label resets, flavor-domain expansion, BBL timing,
custom G-code templates, extrusion coordinates, pressure equalization and all
other writer state. Existing Ares shells for those concepts are not promoted
into an independent pipeline by this slice.
