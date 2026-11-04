# Test retirement — ten obsolete Arachne Filled rejection contracts

Status: accepted.
SPEC: `97aa276f9f2377fd54f1afb024c4ed3491bb0664`.
Frozen public RED: `19eaf9b964ec19c46839a192ca22c38af5eb8869`.
RED evidence / exact test preimage:
`2e56c73756e014f8c1539f5e8e9bc175e3da96a5`.
Main base commit: `5a9687ac020962f254b6262f2184f3345025c70f`.

## Why retirement is necessary, and what it does not permit

Pinned `8500fcdccaa10b5099ac20d252af3a7c560046f1`
`libslic3r/PerimeterGenerator.cpp::process_arachne:2123` calls shared
`process_no_bridge:1728–1929`; Filled behavior is `:1777–1855`.
Arachne plus sacrificiallayer (the Filled alias) is supported by Native, not an
upstream capability rejection. Eight new public complete-byte RED cases now
expose exactly the Rust compatibility guard. The ten older contracts expect
that same temporary guard before various later stages. They must not keep
requiring a feature to be unsupported after implementing its real handoffs.

This is not a comparator/expected-output relaxation or substitution of another
Unsupported. The ten new public non-None eventual PASS assertions and main-owned
None remain unchanged; so do all controls and source inputs. The new public
complete-byte Native acceptance is mandatory after GREEN and independent of
this retirement. It does not prove KsrArchive-specific Native parity or the
still-deferred nested-island/multiregion reachability gaps.

The positive layer-region replacement (case2) preserves that hook's live test
coverage: the identical archive must finish successfully, return nonempty
G-code and invoke the layer-region finish hook exactly once. This hook increments
once at `perimeters/layer_region.rs:42`; the active public orchestration calls
`finish_with_arachne` once at `perimeters.rs:180`. This new success assertion is
written before production, remains RED under today's guard, and is not yet
claimed experimentally GREEN. No counter instrumentation or production-owner
change is proposed. Deleting its only caller would instead strand test hooks
outside the three-file production allowlist.

## Per-case decisions

Each case has an exact diff, rather than generic permission to modify assertions:

| Case | Old test contract | Proposed action |
| --- | --- | --- |
| 1 | `arachne_sacrificial_layer_counterbore_hole_remains_out_of_scope` | Remove that obsolete test + unused SliceError import only. |
| 2 | `task22o16_counterbore_preflight_precedes_layer_region_lifecycle` | Rename to `arachne_filled_counterbore_reaches_layer_region_lifecycle`; same input; unwrap success + nonempty G-code + finish count exactly1 instead of rejection/zero. |
| 3 | `task22o18_earlier_errors_leave_invocations_zero` | Remove only the counterbore rejection loop; preserve the independent O17 injected geometry failure/zero invocation assertions in the same test. |
| 4 | `task22o25_every_earlier_capability_error_precedes_promotion` | Remove this obsolete-only test; preserve separate real parse/geometry failure, commit/disposal/propagation assertions verbatim. |
| 5 | `task22o17_counterbore_fails_before_detection` | Delete the now-empty obsolete-only lifecycle file and its single parent module registration; retain all other detection tests/hooks. |
| 6 | `task22o23_all_earlier_capability_errors_keep_filtering_uninvoked` | Remove obsolete-only test + unused SliceError import; preserve independent geometry and anchor failures. |
| 7 | `task22o20_earlier_errors_have_precedence_and_zero_invocations` | Remove obsolete-only test; retain O17/O19 error/zero invocation and transactional/drop-probe assertions. |
| 8 | `task22o22_earlier_capability_errors_have_precedence` | Remove obsolete-only test; retain inactive typed-mode, predecessor geometry and anchor assertions. |
| 9 | `task22o21_earlier_capability_errors_have_precedence` | Remove obsolete-only test; retain geometry/anchor zero invocation and whole-project transaction/drop assertions. |
| 10 | `task22o19_earlier_capability_errors_have_precedence` | Remove obsolete-only test; retain O17 actual geometry error and zero-cache assertion. |

Exact paths, old/new lines, imports and module deletion are fully enumerated:
- [Cases1–5 exact diffs](arachne-counterbore-test-retirement-diffs-1-5.md)
- [Cases6–10 exact diffs](arachne-counterbore-test-retirement-diffs-6-10.md)

Those fenced diff blocks concatenated in case order are the entire proposed
patch.
SHA256: `1de1f3ee800372058543017faede48a3ce8e60096bb69c7e3150d14813e13972`.
`git apply --check` PASS on the named preimage, **without applying it**.
All proposal generation is read-only against repository test sources.

## Tight execution boundary

- Eleven test-only paths: the ten original files, plus detection's parent module
  registration. Eight obsolete-only tests are removed, one is a success migration,
  and one mixed test keeps its real geometry-error assertions. No empty loop/file,
  ignored test, dead import or fake guard is retained.
- No unrelated tests, fixtures, error strings, disposal/transaction/commit or
  timing assertions are modified. Zeros retired here belong exclusively to the
  now-supported counterbore input, not to the independent genuine error cases.
- Shell and classification production code remains read-only. Changing tests under
  those directories does not permit changing those algorithms.
- Commit this exact test change separately **before** production. The public RED
  commit stays untouched. No production change is hidden in a test commit.
- Then implement GREEN only in `classic/preflight.rs`, `classic/counterbore.rs`,
  `arachne/materialize.rs` under `ares-core/src/project_slice/perimeters/`.
  Anything more requires a scope amendment.
- Fresh Native caches/artifacts. After GREEN verify the success replacement and all surviving
  affected lifecycle/error/transaction tests, plus both producer public complete
  output and control raw-SHA invariance. If a new mismatch appears, stop and
  report; no assertion drift or wider implementation by inference.
- Strict clippy/rustfmt and core/adapter/vgcode WASM checks follow. Full parity
  still needs actual evidence.
