# Classic Filled GREEN — bounded public parity evidence

Status: directed GREEN validated; no full, merge approval or complete Filled
coverage claim. SPEC/RED and assertion migration were coordinator-approved.

Production base: `023b742f26caab6c51df9ebb6db156d44792fc4b`.
Rebase from `6a6587f8f` preserved all five branch commits (`range-diff` all `=`),
receipt `/tmp/worker4-counterbore-fuzzy-range-diff.log`.
Original RED `8cd21977d` is frozen, now cleanly carried as `08a131e0d`.
Mechanical runner split is separate (`016639259`); approved assertion migration
is separate (`ab7d89e87`, approved patch SHA256
`539b685660329a8755a8c7dccad77a9ad58adf19deeb14e865a4eef8030a601f`).
The production GREEN commit must be reviewed together with that test commit.

## Upstream rewrite implemented

Pinned OrcaSlicer source remains `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
This is an extension of the existing upstream-boundary port, not an Ares-owned
bridge feature or G-code patch:

- `PerimeterGenerator::process_no_bridge`, `PerimeterGenerator.cpp:1777–1855`:
  Filled candidate shrink/convexity/bridgeability/support filtering, anchor
  restoration and clamping, unsupported-hole removal and nested-island mutation
  logic. Rust owner: `perimeters/classic/counterbore/filled.rs`, invoked before
  wall generation from existing `classic/counterbore.rs` / prelude.
- Common gates, detector and shared fill/remainder handling remain in the
  existing port (`:1731–1775,1899–1929`). None and classic Bridges retain their
  existing computation. The margin is the named upstream
  `bridged_infill_margin = scale_(BRIDGE_INFILL_MARGIN)` (`:1736`;
  `libslic3r.h:89`), with `min(margin, spacing, external_width)` anchor (`:1774`),
  as confirmed in the approval clarification; no config option was invented.
- `Polygon.cpp:107–123,235–255,315–317`: reuse polygon simplification; the
  candidate convexity predicate ports zero-threshold concave-vertex detection.
- `Layer::make_perimeters`, `Layer.cpp:248–279`: use existing group
  redistribution/extra-fill recovery, not another stage or state model.
- `PrintObject::detect_surfaces_type`, `PrintObject.cpp:1579–1583,1698–1704`:
  only Filled expands the top/bottom comparison domain using generated fill.
  Internal surfaces still derive from original slices. No-upper/no-lower
  branches retain upstream original-slice handling. Existing typed fill clipping
  continues to port `LayerRegion::slices_to_fill_surfaces_clipped:63–80`.
- Remove only classic Filled's temporary compatibility rejection. Arachne's
  non-None hole guard remains explicit; no fallback, no silent skip.

## Public GREEN and non-Filled invariance

Fresh valid Native evidence root:
`/tmp/worker4-counterbore-filled-green-e5ml`.
Official and seeded have independent `cache/`, `artifacts/`, `run.log`, `exit`.
Compiled target only was reused from `/tmp/worker4-counterbore-filled-red-bGXN`.
All cargo/nextest/Native execution inherited outer `taskset -c 15`, with
jobs1 / test-threads1 / incremental0 / Rust 1.99.0. START/END in coordinator
inbox; no full/workspace tests were run.

| Directed gate | Result |
| --- | --- |
| official public counterbore matrix | 11/11 PASS, 10.146s |
| seeded public counterbore matrix | 11/11 PASS, 12.567s |
| Existing non-Filled controls, raw Ares SHA vs frozen RED | 6/6 identical in each mode |
| Approved Arachne capability/precedence guards | 10/10 PASS, 9.112s |
| `cargo fmt --check` | PASS |
| `cargo clippy -p ares-core -p ares-cli --all-targets --all-features -- -D warnings` | PASS |
| `cargo check -p ares-core --target wasm32-unknown-unknown` | PASS |

The five new **classic Filled** cases use public `ares_core::slice_with_profiles`
and complete Native-backed ordered G-code assertions: primary stepped hole,
first-layer hole, straight hole, solid plate and two independent islands.
Primary, first-layer, straight-hole and two-islands changed from exact guard RED
to complete-output PASS. Filled solid remains PASS. This is not a success-shaped
private polygon test or an assertion of mere nonempty output.

Existing classic partial/None controls are unchanged. Invariance receipt:
`non-filled-invariance.json` under the valid root, comparing exact Ares SHA256
against `/tmp/worker4-counterbore-filled-red-fbNb` (six cases per mode). No M73,
movement, statistics/footer lines were removed. Native/Ares parity uses only
established generator identity normalization.

Approved lifecycle migration changes only the wall generator to Arachne for
10 existing guard cases. Same literal counterbore error and all existing
0-invocation/disposal/commit assertions remain. `guard-tests.log` proves these
precedence contracts still pass; they do not stand in for classic Filled parity.

Official wrapper SHA256:
`1689a2ff46564cf723ad90c663051b58115920c3cce639bd2961340955cebbf3`.
Native uses `setarch -R`; seeded also uses seed0/CPU15. No Variant A or other
Native mismatch occurred in this final directed matrix.

Additional gates also passed in the same root:
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  (`workspace-clippy.exit = 0`);
- `cargo check -p ares-wasm -p ares-vgcode --target wasm32-unknown-unknown`
  (`wasm-adapter.exit = 0`).
These are compile/lint gates, not workspace tests. Final changed Rust modules
are below 400 LOC: runner 322, mechanical helpers 85, counterbore 217, Filled
162, surface classification stage 303.

## Reproduction, failures and coverage limits

Re-run the RED document's public matrix command at the GREEN commit with fresh
mode caches/artifacts and CPU15. The exact directed + selected-guard + gates
launcher is preserved as `/tmp/worker4-counterbore-green-final.sh`.
Mode producer inputs follow the unchanged control-set-v2 binary/resources
selection; no full control script was invoked for directed evidence.

Earlier failures remain recorded, not silently replaced:
- `/tmp/worker4-counterbore-filled-green-XlIF`: compile E0432 from an incorrect
  simplification import; no Native comparison executed.
- `/tmp/worker4-counterbore-filled-green-gAOx`: both public modes passed, but
  strict found two functions over the repo's five-argument limit. Grouping the
  existing traversal/domain arguments fixed it without a lint allowance; the
  final valid run revalidated public parity and strict.
- Extra gate launcher initially lost cargo from PATH in an `env -i bash -c`
  shell. Exit127 logs remain as `*-envfail.*`; corrected script-file execution
  uses an explicit cargo path. No production fallback was added.

**Coverage gaps remain:** no frozen public nested-island-in-hole fixture or
compatible multi-region fixture proves those particular recovery branches.
The implementation ports their source logic, but the two-independent-islands
case does not establish nested-island coverage. Do not claim complete Filled
coverage from this matrix. Follow-up Native-backed reachability work remains
required for those branches, as accepted by coordinator for this GREEN slice.

Arachne counterbore remains DEFERRED and is W4's next separately specified
slice, covering `process_arachne:2123` and both non-None modes. Unsupported
interface-shell/spiral/modifier/raft/multi-material domains are not expanded
here. No self-designed bridge pipeline, changed threshold, new ignore,
assertion relaxation or Native binary commit was introduced.
