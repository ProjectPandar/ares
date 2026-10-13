# C2 additional lifecycle guard retirement request

Status: separately approved by the coordinator on2026-10-14 and applied
in a separate test commit; not part of the bb886ec6c approval.
RED4334347e4 and both-mode evidence:
`/tmp/worker3-centering-c2-red-matrix-izu0d3pw/red-evidence.json`.
Source: `bbs_3mf.cpp:4931-4974,5025-5052`; `Model.cpp:1295-1307`.

The final whole-test-tree search finds one additional old shared guard
expectation in `project_slice/tests/raw_lifecycle.rs`: the assert_eq block
using `invalid_coordinate.clone(), shared_later` expects
`unsupported("shared_mesh_centering")` before coordinate validation.
This assertion was not listed in the first retirement request and is unchanged.

Request deletion only of that assert_eq block and its exclusive `shared_later`
fixture, in a separate test commit (or separately approved amendment before
GREEN). Preserve the preceding coordinate_error assertion and the following
dense precedence assertion, all other lifecycle assertions and test names.
No replacement assertion or error-order normalization. Private fixture bool
arguments were already mechanically removed under the first approval.
