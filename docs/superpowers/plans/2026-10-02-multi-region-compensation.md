# Multi-region compensation execution plan

## Source slice and destination

`PrintObjectSlice.cpp:1314-1365` and `Layer.cpp:119-138` own the branch port
into `ares-core/src/project_slice/compensation/multi_region.rs`.
`PrintObjectSlice.cpp:1512-1544`/`ClipperUtils.cpp:274-305::raw_offset` own the
existing independent contour/hole and clockwise-offset dependencies.
ARD0088 records non-negotiable geometry/ownership and cache-lifetime limits.
The single-region shell remains temporary; no independent Ares algorithm is
introduced. The included/deferred boundaries are in the accompanying spec.

## Sequence

1. Retain public modifier RED, then supplement with shared-boundary geometry.
2. Port the full source positive/negative/elephant-foot branches before
   removing compensation's guard; preserve single-region output.
3. Cover priority, subtractors, first-trigger radius and mixed signs;
   independent six-axis review and repair newly introduced warnings.
4. Isolate only this prerequisite on201a0782, excluding Normal dependencies.
5. Run clean workspace, public native controls, Tier1 compilation and browser
   parity; preserve failures and record exact producer limitations.
6. Freeze the exact patch for independent six-axis review. Commit/push code
   and documents together only if that narrow preserving slice is approved.

## Retained development evidence

In the unmerged Normal/Tree worktree, the first adjacent-region elephant-foot
tracer failed at the previous compensation guard, then passed after the port:
`/tmp/ares-multi-region-compensation-foot-red.log` exit100 and
`/tmp/ares-multi-region-compensation-foot-green.log` exit0.
Four supplementary geometry cases and all23 compensation tests pass after
review repairs (`/tmp/ares-multi-region-compensation-after-review.log`,43.554s).
Review caught five new nesting warnings; focused single-region and positive
allocation functions removed them without changing source order.
Clippy exits0 with no compensation-source diagnostics at
`/tmp/ares-multi-region-compensation-clippy-after-review.log`.

The mixed-sign supplemental case initially used an incorrect hand expectation
that negative hole compensation expands a CW hole. Native `raw_offset`
negates the CW offset, so it shrinks the trimming hole; final intersection
cannot fill the larger hole already retained by a region. Corrected only
that worked expectation; production code is unchanged. Initial failure is
retained in `/tmp/ares-multi-region-compensation-mixed-sign.log`; corrected
run exits0/1PASS in0.025s at
`/tmp/ares-multi-region-compensation-mixed-sign-corrected.log`.

The unmerged tree's final workspace exits100:7,483run/7,482PASS/one original
KSR Tree-width FAIL/nine skipped/254.545s. This is not isolated-patch evidence.
The unchanged public modifier E2E still fails, with baseline PASS and modifier
ARES_ERROR at the downstream perimeter limit; artifacts are retained under
`/tmp/ares-multi-region-compensation-modifier-output`. Cache manifest producer
identity remains unknown, despite a separate successful official command
journal. No reference cache/scheduling patch or variant selection is approved.

## Isolated verification checkpoint

Worktree `/tmp/ares-compensation-isolated.wTtpf4`, base201a0782.
Prepared only compensation code/tests and these source-cited documents.
Excludes Normal raw slices, support code/scheduling and slicing parameters.
For the strict400LOC boundary, moved the unchanged existing public raft test
out of `gates.rs` into its own module; assertions/fixture remain unchanged.

## Terminal isolated verification

- Workspace `/tmp/ares-compensation-isolated-workspace.log`:exit0,
  7,469run/7,469PASS/nine skipped/37slow/265.623s. All five supplementary
  multi-region examples PASS. Existing KSR complete initialized-reference
  ordered-byte test PASS in42.391s; this does not prove actual enabled Tree.
- Clippy `/tmp/ares-compensation-isolated-clippy.log`:exit0,5m33s,
  268core-test warnings/252duplicates, no compensation-source diagnostics.
  fmt/diff checks exit0. All changed source modules remain below400LOC.
- Windows GNU core check exits0/6m07s; macOSx86 core check exits0/6m28s.
  These are compile checks, not Windows/macOS runtime proof. Logs use
  `/tmp/ares-compensation-isolated-{windows,macos}.log`.
- WASM release exits0/4m23s; binding generation exits0 into
  `/tmp/ares-compensation-isolated-bindings.HJUOH8`. Bindgen WASM SHA256
  `53cfa14f3ed59201358fd55ab0cca2930ab1ab1bfa73bf706793bd4b2eeb18cd`.
- Official controls fixed `ARES_ORCA_RUNS=1`, explicit untouched extraction,
  fresh output root, no variant-selection retry. Ender-3 and concentric both
  PASS,exit0/0.887s, at
  `/tmp/ares-compensation-isolated-official-single-run.log` and its
  `-output/case-QgZu0k`/`case-6ndMpq` artifacts. Manifests still report cached
  producer identity unknown. Separate command/status journals show explicit
  official extraction and successful export/slice processes; this supplements
  evidence, not a provenance-classification repair.
- Actual Chromium143.0.7499.4 matches complete Ender-3 browser/host/native
  streams except generator line, nonfinite motion false; probe exit0 at
  `/tmp/ares-compensation-isolated-browser.log`. Full artifacts under
  `/tmp/ares-compensation-isolated-browser-output.dIzReT`; browser SHA256
  `70a03541736e97d5a19fded3e10a9e413642128ca64abc16074a5205c035a577`.
  The probe uses this worktree/server/new WASM, injects only input and bindings,
  not G-code. This is existing single-region output preservation, not browser
  execution of completed modifier/Tree output.

Read-only comparison with prior independent-islands controls confirms exact
same normalized Ares Ender-3 and concentric outputs at
`/tmp/ares-compensation-isolated-preserved-public-output.log`. Concentric's
uncompressed project payload hashes also agree. The older one-run native
concentric divergence remains retained and unresolved; this fresh PASS cannot
close it or establish a noise/undefined-field cause. No extra native attempt
was used to pick a favorable reference.

Terminal finite checks are complete. Immutable-patch review/submission are
pending at this checkpoint. Normal/Tree, native cross-job/parallel epsilon
semantics and all-printer/option acceptance remain open.
