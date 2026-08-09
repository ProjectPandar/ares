# Task 22O.36 — Bridge-zone anchor and ExPolygon expansion plan

## Goal and release baseline

Tasks 1-10 are complete. The compiling empty-stub RED failed 0/6; the frozen
candidate passes 6/6. Pinned original-Orca E2E and byte-identical Debug/NDEBUG
helper output, 13 runtime mutation kills, two truthful equivalent survivors,
two compiler rejections, exact restoration, complete native/WASM/static/
rollback verification, and both final review rounds pass. Implementation and
documentation commits `b546e6f`/`3e927ed` were pushed; exact-SHA Tier-1 run
`31280579891` passed all five jobs and both browser executions at
`3e927ed569d3db8d6f5c08b7843fb049fcc86412`. O36 remains inactive.

Port only pinned OrcaSlicer v2.4.2
`LayerRegion.cpp:353-356,358-393`: the translation-unit-local
`ExpansionResult` and `expand_expolygons` helper. This is not released O32's
`Algorithm::expand_expolygons` and does not activate external-surface slicing.

Released predecessor O35 is implementation/documentation commits
`984bc01`/`c6f23ce`. Exact-SHA Tier-1 run `31269521736` passed all five jobs and
both browser executions at
`c6f23ce1a9350ca76241d007f804f3fcfa22c352`; authoritative JSON is
`/tmp/task22o35-tier1-exact-sha.json`.

O36 composes released O28 `wave_seeds` and O30 `propagate_waves_ex` across O35
`ExpansionZone`s. It remains crate-private and inactive. Bridge grouping and
direction helpers, `process_external_surfaces`, lifecycle, Options, fill,
toolpath, motion, serialization, and G-code remain deferred.

One delegated worker session is the sole writer for every Rust/test byte,
including witness repairs, one-at-a-time mutations, exact restoration, and all
review-requested Rust/test fixes; revive that same session rather than starting
a second writer. The parent never edits Rust/tests. The parent runs commands,
diagnoses and authorizes RED/GREEN, writes approved documentation/evidence,
commits, pushes, and verifies exact-SHA CI. Reviewers are read-only.

## Exact allowlist

Rust edits only:

1. `crates/ares-core/src/geometry.rs` — crate-private `wave_seeds` facade
   reexport and one matching function-shape assertion only.
2. `crates/ares-core/src/project_slice/prepare_infill/external_surfaces.rs` —
   module/reexports and exact shape assertions.
3. `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/types.rs`
   — `ExpansionResult` only.
4. New
   `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/expand_expolygons.rs`
   — sole production body, at most 150 physical lines.
5. `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/tests.rs`
   — one ordinary module registration.
6. New
   `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/tests/expand_expolygons.rs`
   — focused tests, at most 300 physical lines.

Documentation edits only: this spec/plan, O35 spec/plan release-state
corrections, `docs/roadmap.md`, and
`docs/architecture/option-parity-v4.md`. Every Rust file stays below 400 LOC.
`.pi-subagents/` remains a known untracked and unstaged local exception; reject
every other unexpected tracked or untracked path.

## Task 1 — Freeze source and baseline

1. Record `HEAD == origin/main == c6f23ce...`, a clean index, the allowed
   untracked O36 documents, and the known `.pi-subagents/` exception.
2. Inspect the pinned C++ helper and current O28/O30/O35 Rust APIs. Record that
   `wave_seeds` is crate-private but not sibling-path-reachable through
   `geometry.rs`; do not edit its kernel.
3. Capture baseline debug tests: O35 13, O28/O30/O31 focused suites, complete
   RegionExpansion 92, PolyTree 6, offset 62, and O26 lifecycle 3.
4. Run pinned Orca itself from a disposable exact-source worktree. Use:

   ```text
   git -C OrcaSlicer worktree add --detach /tmp/task22o36-orca \
     8500fcdccaa10b5099ac20d252af3a7c560046f1
   cd /tmp/task22o36-orca
   ./build_linux.sh -g -dstrlL
   ORCA_BIN="$(find build -type f -perm -111 \
     \( -name OrcaSlicer -o -name orca-slicer \) | head -1)"
   mkdir -p /tmp/task22o36-orca-e2e
   "$ORCA_BIN" --slice 0 --outputdir /tmp/task22o36-orca-e2e \
     /home/indexyz/ares/tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf
   ```

   Archive stdout/stderr, exit code, `result.json`, generated-file names/sizes,
   and the disposable worktree SHA under `/tmp`; require exit zero and a
   nonempty generated G-code. Do not read that generated G-code from Ares code,
   copy it into the repository, or use it as a runtime oracle.
5. Because the O36 helper is translation-unit-local and the normal CLI does not
   expose its intermediate anchors/expansions, also build a disposable pinned
   C++ oracle in that exact worktree. Add a temporary test-only wrapper beside
   `LayerRegion.cpp:358-393` in the disposable worktree, invoke it with the same
   behavior-named zone/source vectors used by Rust, and serialize only ordered
   seed paths, expansion contours/holes, IDs, and flags to
   `/tmp/task22o36-orca-helper-{debug,ndebug}.txt`. Build/run both Debug and
   `NDEBUG`; require byte-identical outputs, then manually compare every
   committed complete Rust literal. The wrapper must call the original
   `wave_seeds` and `propagate_waves_ex`; it must not reimplement either kernel.
   Remove the disposable worktree afterward. No harness, patch, raw output, or
   blob is committed.
6. Archive exact commands and outputs under `/tmp`; never commit generated
   output or oracle blobs.

## Task 2 — Add a compiling stub and capture chronological RED

The sole writer adds only the approved API/reexports/assertions, ordinary test
registration, and a temporary body returning empty vectors:

```rust
Ok(ExpansionResult {
    anchors: Vec::new(),
    expansions: Vec::new(),
})
```

The result and entry are exactly:

```rust
pub(in crate::project_slice) struct ExpansionResult {
    pub(in crate::project_slice) anchors: Vec<WaveSeed>,
    pub(in crate::project_slice) expansions: Vec<RegionExpansionEx>,
}

pub(in crate::project_slice) fn expand_expolygons(
    expolygons: &[ExPolygon],
    expansion_zones: &mut [ExpansionZone],
    scale: CoordinateScale,
) -> Result<ExpansionResult, ClipperError>;
```

The focused shard covers:

- zero zones and exact result shape;
- empty source with multiple zones and positive tiny-expansion preconditions;
  pre-set every `expanded_into` flag to `true` so successful visits resetting
  flags to `false` are observable;
- one natural source/zone with complete seed and expansion literals;
- multiple ordered sources/zones, a leading or interior empty-output zone,
  complete append order, holes, point order, and global boundary rebasing by
  every prior zone's full ExPolygon count;
- equality to an explicit per-zone O28→O30 pipeline plus at least one
  independent complete literal;
- Normal and LargeBed vectors;
- first- and later-zone discovery/propagation errors, exact flag state, and no
  partial returned result;
- sorted discovery and O30 propagation/assertion ordering.

Run and archive:

```text
cargo nextest run -p ares-core task22o36
```

Require successful compilation and meaningful failures at the stub seam.
Truthfully disclose stub-equivalent zero-zone/function-shape behavior and any
other survivor. Function pointers are type evidence, not RED. Do not install
the production body until the parent authorizes GREEN.

## Task 3 — Install the frozen source-shaped body

Replace only the stub with:

```rust
let mut anchors = Vec::new();
let mut expansions = Vec::new();
let mut processed_bridges_count = 0_u32;
for zone in expansion_zones {
    let mut zone_anchors = wave_seeds(
        expolygons,
        &zone.expolygons,
        zone.parameters.tiny_expansion,
        true,
        scale,
    )?;
    let mut zone_expansions =
        propagate_waves_ex(&zone_anchors, &zone.expolygons, &zone.parameters)?;
    for anchor in &mut zone_anchors {
        anchor.boundary = anchor.boundary.wrapping_add(processed_bridges_count);
    }
    for expansion in &mut zone_expansions {
        expansion.boundary_id = expansion
            .boundary_id
            .wrapping_add(processed_bridges_count);
    }
    zone.expanded_into = !zone_expansions.is_empty();
    anchors.append(&mut zone_anchors);
    expansions.append(&mut zone_expansions);
    processed_bridges_count = processed_bridges_count
        .wrapping_add(zone.expolygons.len() as u32);
}
Ok(ExpansionResult { anchors, expansions })
```

Run `cargo fmt --all`, focused debug and release, and the O28/O30/O31/O35 and
complete RegionExpansion regressions. Repair only incorrect test witnesses;
never change the frozen body to accommodate a witness.

## Task 4 — Audit semantics and ownership

Audit the exact candidate against every source operation:

1. zone order is untouched;
2. each zone calls O28 once with source, zone geometry, tiny expansion,
   `sorted=true`, and the same scale;
3. each successful discovery reaches O30 once with unchanged zone geometry and
   parameters;
4. seed `boundary` and expansion `boundary_id` receive the cumulative prior-zone
   ExPolygon count before append;
5. `expanded_into` commits only after both fallible calls and rebasing;
6. anchors append before expansions within their separate complete streams;
7. count advances by `zone.expolygons.len() as u32`, even for empty output;
8. errors escape directly, prior flags stay committed, failing/later flags keep
   entry values, and no partial result escapes;
9. source/zone geometry and point buffers remain borrowed and unchanged.

Prove scale-visible behavior with complete dual-scale vectors where possible,
but fix unchanged forwarding through literal-body/diff audit. Report equivalent
scale substitutions and unreachable 32-bit overflow as survivors; do not add a
production injection or allocation seam.

## Task 5 — Run one-at-a-time mutations

The same delegated Rust/test writer applies, tests only when its toolset permits,
and restores each mutation separately; the parent runs every authoritative
command and confirms exact restoration before authorizing the next mutation.
Candidates include:

- omit/reorder zones;
- `sorted=false`;
- substitute source, boundary, tiny expansion, parameters, or scale;
- omit O30 or swallow an O28/O30 error;
- omit one rebase, use current-zone count before rebase, count outputs instead
  of full zone ExPolygons, or use non-wrapping arithmetic;
- invert/omit/early-commit `expanded_into`;
- omit/swap append operations or return partial output;
- alter result fields, signature, or visibility.

Record killed runtime mutations, compiler rejections, and behaviorally
identical survivors separately. Restore exact production/test bytes and rerun
focused debug/release. Mutation evidence is post-hoc and must not be described
as chronological RED.

## Task 6 — Initial independent implementation review

Run in parallel:

1. a read-only independent six-dimensional reviewer covering requirement
   completeness, logic, edge cases, code quality, test coverage, and actual
   results;
2. a default-model OpenCode read-only implementation review over the same diff
   and evidence.

Require literal `VERDICT: APPROVE`. The parent converts every issue into a
repair list, then revives the same delegated Rust/test writer for accepted
Rust/test repairs; the parent alone applies approved documentation/evidence
repairs. The parent reruns affected and complete exact-candidate checks,
refreshes evidence, and requests both rereviews. Repeat until both approve.

## Task 7 — Update truthful documentation

Update O35 release records to implementation/documentation commits
`984bc01`/`c6f23ce`, run `31269521736`, exact SHA
`c6f23ce1a9350ca76241d007f804f3fcfa22c352`, all five jobs, and both browser
executions. Record O36 as locally implemented, crate-private, inactive, and
unreleased pending final review, commit/push, and exact-SHA Tier-1. Name the
next exact pinned source boundary; do not claim external-surface lifecycle or
KSR G-code parity.

## Task 8 — Verify the exact documented candidate

Run and archive, on the exact bytes to be reviewed:

- O36 debug/release;
- O35 13 and focused O28/O30/O31/RegionExpansion suites;
- complete RegionExpansion/external-surface tests, PolyTree 6, offset 62, and
  O26 lifecycle 3;
- `cargo nextest run --workspace`;
- all-target workspace check;
- all-feature workspace Clippy with `-D warnings`;
- `cargo fmt --all --check`;
- all four required wasm32 checks;
- two optimized WASM builds, wasm-bindgen/export audit, and JavaScript syntax;
- the full Playwright suite twice.

If local Chromium cannot load `libglib-2.0.so.0`, record both failures exactly
as environment failures and require both exact-SHA CI browser executions. Do
not call them passes.

Static audit exact changed-path allowlists, ordinary module usage, LOC,
crate-private visibility, absence of `include!`/`include_bytes!`, unsafe/FFI,
filesystem/thread/platform branches, broad lint allowances, source pinning
tests, fixture identity branches, reference G-code reads, dependencies,
adapter/lifecycle/golden changes, staged files, and generated artifacts.

## Task 9 — Rehearse exact-O35 rollback

In a disposable worktree at exact `c6f23ce...`, rerun O35 13, RegionExpansion
92, PolyTree 6, offset 62, and lifecycle 3. Prove the primary worktree's diff,
index, and baseline identity did not change. Delete the disposable worktree.

## Task 10 — Final reviews, commit, push, and exact-SHA gate

1. Run both final implementation/evidence reviews. Apply no silent repairs.
2. Make the final allowed documentation update once; rerun all exact-candidate
   verification and static/rollback identity checks after that byte change.
3. Run both final documentation rereviews until literal approval.
4. Stage only approved files. Use separate Conventional Commits for
   implementation and documentation; verify `.pi-subagents/`, `/tmp`, and
   generated evidence are unstaged.
5. Push and require `HEAD == origin/main`.
6. Wait for Tier-1 whose `headSha` exactly equals the pushed documentation SHA.
   Require exactly five successful jobs and both successful browser executions.
7. Archive post-CI release evidence only outside the repository, for example
   `/tmp/task22o36-tier1-exact-sha.json`. Make no tracked O36 edit after the
   successful exact-SHA run; released-state corrections belong to the next
   bounded milestone. If any tracked byte changes, create a new documentation
   commit, rerun complete exact-byte verification and both reviews, push, and
   require a new matching exact-SHA Tier-1 run. O36 remains inactive; the
   overall KSR G-code objective remains incomplete and continues with the next
   source-cited boundary.
