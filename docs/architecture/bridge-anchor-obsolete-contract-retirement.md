# C2-S main CI obsolete contract RETIREMENT-REQUEST

Candidate: aa6f0488f41d742f3626bdcc8b1609693f22c587, clean C2-S carry
onto 4ca7554c6e729528883f9860ca700f874aeade00. New web/vgcode main
325ea6983 may be carried git-only after the reproduction. No test/source
edits have been applied. Proposed exact diff:
`/tmp/worker2-c2s-obsolete-contract-retirement.patch`, SHA256
`a2556b4e3804868e455391b4b7bef911d15ce22844abd3c1cc39ccdd7a1430ab`.

## 1. Retire unported_anchor_group_params

Target: crates/ares-core/src/project_slice/tests/prepare_infill/
bridge_over_infill/transaction/options.rs:143-155.

The two mutations change GLOBAL top/solid pattern to gyroid and expect
UnsupportedProjectFeature from bridge transaction preparation. They are
not local overrides. Upstream commit
8500fcdccaa10b5099ac20d252af3a7c560046f1:
PrintConfig.cpp:2074-2086 defines eight global top values excluding gyroid;
:2106-2112 copies that subset to internal-solid; :10406-10408 and
:10416-10418 reject invalid global top/solid before slicing. C2-T ports this
boundary to loaded-config validation. Therefore the old stage/error contract
is invalid; transaction::prepare must not be reached with this global input.

Request: retire this obsolete test rather than change it to expect a late
Unsupported error. Existing public tests global_top_grid_has_native_config_error
and global_solid_grid_has_native_config_error in runner/compatible_regions/
global_pattern.rs independently assert Native's exact config diagnostic,
absence of Native output, Ares API error and CLI behavior. Local admission
is covered by unchanged C2-T/C2-S public E2Es. These tests remain untouched.

## 2. Retire unported_anchor_surface_kinds

Target: crates/ares-core/src/project_slice/tests/prepare_infill/
bridge_over_infill/transaction.rs:123-131.

This directly clones a surface as InternalVoid and asserts the removed
bridge_over_infill_anchor_surface_kind guard. Accepted ruling f586ddc40:
upstream PrintObjectSlice.cpp:22 initializes infill_only_where_needed=false;
PrintObject.cpp:3871-3872 skips clip_fill_surfaces; LayerRegion.cpp:954
selects stInternal, not stInternalVoid. Ares external-surface reconstruction
also cannot produce InternalVoid before this transaction (accepted proof in
bridge-anchor-internal-void-reachability.md). The injected private state is
not a reachable public contract and cannot justify restoring an Unsupported
check. Request: retire it, preserving existing real-KSR bridge geometry,
repeatability, empty-inventory and ownership/error tests unchanged.

## Exact bounded patch and verification

Patch removes only these two tests, their now-unreferenced
assert_unsupported_mutation/assert_unsupported_raw helpers, and the resulting
unused helper import. assert_error_mutation/assert_error_raw and all other
assertions remain unchanged. No production change, fallback, suppression,
ignore or expected-count adjustment. After approval, apply as an independent
test commit on C2-S, include source/ruling evidence in docs/architecture and
commit message, then reproduce task22o71 green, run all ares-core tests plus
strict/fmt/wasm, and wait explicit FULL-GO. Core must be green before full.

Both complete option_coverage modes on aa6f0488f passed 220/220:
official /tmp/worker2-c2s-option-coverage-official.qTjxPd;
seeded /tmp/worker2-c2s-option-coverage-seeded.hQadov.
Reproduction: /tmp/worker2-c2s-task22o71-red.leLQ6G, outer taskset13,14,
standard cleared env, jobs2threads2, no Native. Exact results appended below
when complete. Approval is required before applying the patch, per the
coordinator's explicit MAIN RED instruction.

## Completed exact reproduction

17 tests run: 15 PASS, 2 FAIL, exit100, 78.679s after compilation.
Only the two CI-reported obsolete contracts failed:

1. Group params fails at vertical_shell_projection/fixture.rs:19:10:
   `InvalidValuesInProject("top_surface_pattern: invalid value gyroid")`
   from the fixture unwrap. The first mutation stops at global config
   validation; the second solid mutation is consequently not executed by
   this test. Its schema proof is separate, not a claimed execution result.
2. Surface kinds fails at transaction.rs:196:13, inside the helper's
   success arm: `deferred bridge behavior must not use a fallback`.
   transaction::prepare actually succeeds and is disposed; this panic is
   the retired test's expectation, not evidence of production fallback.

All remaining 15 task22o71 tests PASS, including real committed bridge
geometry, byte repeatability, empty candidate preservation and late error
ownership/disposal. Original logs preserved unchanged. No tests patched.

## Accepted retirement

Coordinator RETIREMENT APPROVED exact patch SHA256 a2556b4e3804868e455391b4b7bef911d15ce22844abd3c1cc39ccdd7a1430ab.
Apply the bounded test/helper/import deletion above as an independent
C2-S test commit. This authorization supersedes the pending-request status
above. Ruling f586ddc40 establishes the public InternalVoid reachability
boundary; this commit does not add a replacement private-state contract.
Production and all retained assertions remain unchanged. Full core and
strict/fmt/wasm evidence must be recorded after git-only carry to 325ea6983.
