# Object Zag ironing admission: Native-first RED

## Authority and source boundary

The NO-DISCRIMINATION result at
`3248dc0f6ee2040b0d9b28df8318c1ff8ca34298` is this chain's spec and covers
three-pattern RED/GREEN on top of Octagram. It supersedes the spec's former
STOP/body-discrimination prerequisite for this slice only. The prior
192-slice result remains true: these are source-defined equivalent ironing
paths, not evidence of a new Zag algorithm or a body-discriminating witness.

Upstream pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `src/libslic3r/Fill/Fill.cpp:1619-1640,1705-1731` initializes density 1,
  monotonic true, and calls the selected filler's Polylines `fill_surface`.
- `Fill/FillBase.cpp:73-75`, `Fill/FillRectilinear.hpp:193-224`: all three
  factories produce classes inheriting `FillRectilinear`'s Polylines path.
- `Fill/FillRectilinear.cpp:3387-3399,2853-2875`: full density goes through
  `fill_surface_by_lines`; the monotonic path is selected. The alternative
  consistent-pattern traversal at **2874 is not reached**. LockedZag's
  separate extrusion override is not called by ironing.
- `Fill/FillBase.hpp:48-109`: initialized `pattern=ipRectilinear`,
  `locked_zag=false`, `horiz_move=0`, multiline 1, `dont_adjust=true`.
- `Print.hpp:347`, `Fill/Fill.cpp:1624,1637`: real object bbox lifecycle,
  independent of the support/raft dummy bbox; no new bbox source/alignment.

Rust destination for the eventual GREEN is exclusively the existing object
ironing admission/dispatch in
`crates/ares-core/src/project_slice/fill_entities/ironing.rs`: fold guard and
pattern dispatch into one `match`; route all three Zags to the existing
rectilinear ironing branch. Preserve existing concentric/PlanePath behavior
and parameters. Cite the upstream inheritance and unreachable 2874 branch.
No second generator, additional algorithm, bbox producer, parameter family,
fallback, or estimator/comparator repair. Ordinary infill, nonmonotonic Zag
traversal, LockedZag extrusion grouping, support/raft and unrelated lifecycle
behavior remain deferred. Existing Ares scaffolding remains a compatibility
shell around the cited upstream call boundary, not a new pipeline.

## Frozen input-only cases

`manifest.json` includes each full archive SHA256 and every member SHA256.
The three inputs are unchanged copies of the earlier remaining-pattern Native
admission discovery inputs: the same 10×10×1mm model and settings, differing
only in `ironing_pattern`. Each prints five model layers with `topmost`
ironing active only on the last layer (268 positive XY/E ironing moves).
There is no committed Native output. Larger non-convex/holed Native evidence
is retained in the accepted spec, not mislabeled as discrimination coverage.

Public E2E seams: `slice_project_plate_validated` plus the actual `ares slice`
CLI, compared against freshly run Native. Tests are
`orca_parity_object_zag_ironing_{zigzag,crosszag,lockedzag}_target`.

Failure modes, established before adding this test code:

1. Corrupt input or changed settings: frozen SHA/config assertions fail.
2. Native rejection, empty activity or config substitution: fail before any
   Ares call. Require positive model extrusion on all five layers, real
   ironing only on the final layer, and retained pattern/type configuration.
3. Nondeterministic Native: run an independent second process and compare
   complete streams, including config/M73/order, normalizing only the existing
   generator-identity line with `GeneratorKind::Orca` on both streams.
4. Ares Unsupported/rejection: **genuinely FAIL** the success expectation,
   after Native evidence and an actual CLI invocation are retained. Never
   assert rejection as a passing result; never fabricate paired target G-code.
5. Admission with output drift: require unchanged
   `golden::compare_ordered_bytes_generator_only` over the complete Native/API
   and Native/CLI streams, retaining config, all toolpaths, M73 and order.
6. API/CLI inconsistency: require raw byte equality and CLI success.

RED validation selects the three new target legs plus the existing Octagram
rectilinear and disabled controls. Expected RED is **3 authentic admission
FAIL + 2 complete-output PASS in each mode**; successful control raw API/CLI
bytes must remain equal to the Octagram GREEN output. Do not treat unexpected
failure, timeout, missing resources, compile/audit failure or skipped Native
as an authentic RED. Native repeats precede public calls per test. Use fresh
serial official and seeded roots/caches.

After GREEN, run all legs and targeted Hilbert/Arch/ironing/infill regressions
in both modes, plus fmt, strict clippy and WASM checks. Keep successful
controls and Hilbert/Arch/Octagram raw outputs stable. Any residual
complete-output difference is STOP/report, not permission for other repairs.
