# Odd-order mesh Native diagnostic premise

## Owning source and destination

This test-only slice follows upstream commit `345efec64c7b95a05ce5cf96df0dde66a158835c`:

- `src/libslic3r/Utils.hpp`, `CLI_SLICING_ERROR`, defines the slicing-failure result code.
- `src/OrcaSlicer.cpp`, `cli_errors[CLI_SLICING_ERROR]` entry near line240 and the `record_exit_reson` / `flush_and_exit` slicing-error branches near7158/7170/7273–7284, own structured error reporting and process termination.
- `src/libslic3r/Exception.hpp::SlicingErrors` names the aggregate exception `Errors`; that is not a guarantee that every slicing failure writes that text to stderr.

Rust destination is only `crates/ares-cli/tests/orca_parity/runner/mesh_odd_order_admission.rs::reject`, reached by the two existing `case!` invocations `odd_reverse_rejection_control` and `odd_rotated_requires_empty_first_layer_error`. The macro creates the existing async tests; no new slicing API or algorithm belongs to this slice.

## Problem and hard boundaries

Both tests currently require an exact aggregate-exception stderr snapshot and stored Native exit values before reaching their strict Ares admission assertions. Live Native structured results establish slicing failure without that stderr spelling. Native return-code equality alone does not establish that a mesh has an empty first layer: a generic slicing-failure code can also represent later empty-layer or other export failures.

The older `mesh-odd-order-layer-state-localization-spec.md` and `mesh-edge-odd-rotated-export-gap.md` retain their historical observations. Do not silently reinterpret those observations as current state. In particular, current diagnostic runs show Native rejection while public Ares API/CLI accept both inputs. This document does not attribute the geometric cause or prescribe an admission guard.

Included behavior:

1. Read the named Native slicing-error code and CLI error-map message from the pinned source, using the same source-derived premise as `mesh_edge_pairing::rejected_orders_diagnostic_only`.
2. Require `FailureKind::Process`, exact Native structured result code/message and the matching captured OS status. Unix status uses the ABI's low eight bits; Windows retains the signed code. No newly stored Native exit expectation.
3. Keep original Native no-Gcode artifact checks and authoritative stdout/stderr/status captures. Do not assert an empty stderr snapshot or treat arbitrary stderr text as the rejection reason.
4. Keep both original fixture identities and labels, full API/CLI invocations, artifact recording, exact `SliceError::InvalidInput` assertion, existing CLI error/message/no-output assertions and test names unchanged.

Deferred behavior: geometry localization, sort or edge-use ordering changes, line/loop construction, new layer-admission guards, modifications to the required Ares error, CLI exit redesign, fixture replacement and output normalization. Existing Rust mesh topology/chaining and project-slice transport are temporary representation shells around the previously cited `TriangleMesh` / `TriangleMeshSlicer` boundaries, not an independent Ares pipeline. None changes here. A current geometry comparison needs a separate source-cited boundary and cannot be inferred from a generic slicing-error code.

## Atomicity decision

The obsolete assertions live once in `reject`; changing that body necessarily changes both Native preconditions. The macro itself need not change. There is no honest single-test behavior change through this helper without adding a temporary label/flag-dependent old expectation or duplicating the complete harness. Neither a legacy assertion path nor a fixture-dependent conditional is acceptable.

One test-only change to the shared `reject` helper covers both tests. Each test is run separately and as a pair. The macro and strict Ares assertions remain unchanged. No compatibility branch or duplicated harness is introduced.

## Tests

1. Preserve an original-source official/seeded baseline for each existing test and the combined pair. Record actual nextest statuses and original failed assertion locations, not capture-wrapper statuses.
2. After the sole Native-premise correction, execute each original test separately and the pair in both Native modes. Ares admission/error assertions stay exact. If either now fails at Ares acceptance or a different typed error, retain the failure as a genuine comparison failure; do not loosen the assertion or report parity success.
3. Bind original input hashes, full Native JSON, process stdout/stderr/status aliases, Ares API results and CLI raw products. Check unchanged products for matching input/provider identities. Structured fields such as runtime timings remain original observations, not new goldens.
4. Independently verify that only `reject`'s Native premise and source lookup change. Macro, labels/fixtures, all existing Ares comparison code and other production/Cargo/API bytes remain identical. Run rustfmt, strict workspace/all-target Clippy, WASM and diff checks.
5. If a genuine comparison is exposed, publish the retained failure and obtain a separate exact source/destination specification before localization or production changes. Test-only diagnostic success is never whole-output or admission parity.

## Failure modes

1. Replacing `Errors` with a newly stored empty stderr, numeric exit or JSON-message snapshot only moves the stale golden. Derive Native values from the named upstream source instead.
2. Source lookup failure is a failure, not permission to use a hardcoded or legacy expectation.
3. An unrelated Native failure must not satisfy the witness: process failure, named slicing-error structured fields, status and missing output all remain required.
4. Generic slicing failure must not be labelled proof of empty-first-layer geometry. Preserve the distinction between structured CLI reason and geometric attribution.
5. Current Ares acceptance must not be hidden by deleting, skipping or weakening the original typed API/CLI rejection comparisons.
6. Both macro-generated tests consume the helper. Do not claim a one-test-only commit for a two-test behavioral change, add label-specific compatibility switches or retarget a fixture.
7. Different Native result timing values or additional JSON observations must remain intact; do not normalize them into invented equality or drop aliases.
8. Preserve all unsuccessful setup, proof-reader and test attempts. An expected failed comparison remains a failed comparison, not a successful parity result.
