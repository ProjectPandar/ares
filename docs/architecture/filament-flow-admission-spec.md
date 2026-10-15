# SPEC request: loaded filament-flow-ratio validation discovery

Status: docs-only SPEC-REVIEW-REQUEST. No Native execution or implementation
is authorized by this document. Stacked on approved clearance GREEN
`df5f13e92def40a5f9ecdbd4b0dc462ce6875768`; wait for broadcast MAIN-ADVANCE
and rebase without stash before any newly approved runtime work.

## Upstream boundary and Rust destination

`OrcaSlicer/src/libslic3r/PrintConfig.cpp:2315-2325` defines
`filament_flow_ratio` as nullable coFloats, default 1, min 0, max 2.
`PrintConfig.cpp:10449-10454`, in `Slic3r::validate`, iterates the vector;
the first element <=0 inserts one `filament_flow_ratio` diagnostic containing
`invalid value ` plus serialization of the WHOLE vector, then breaks.
This is configuration admission, not a new extrusion pipeline.

Rust `options/validation/flow.rs::validate_filament_flow_ratio_options`
already implements the nonpositive-element predicate and vector diagnostic.
The prospective consumer is `options/validation/loaded.rs::
ProjectSettingsRaw::loaded_range_options`, normalized map, inside the existing
`validate_print` branch alongside bridge and clearance validators. Preserve
`merge_first`, BTreeMap ordering, `InvalidValuesInProject`, and CLI -18/238.
Do not activate the aggregate validator or move validation into pure processing.
These existing helpers are temporary compatibility shells for the cited upstream
boundary, not Ares-owned policy.

There is an important preflight boundary to investigate:
`options/flow_ratios.rs::parse_filament_flow_ratio` rejects nonpositive values
with `InvalidInput("filament_flow_ratio contains invalid value")` and is used
by flow construction. Discovery must determine whether that error occurs before
loaded validation for the public validated entry. Different error type/map/exit
from Native is a real admission divergence, not equivalent rejection.
This SPEC authorizes discovery only; if typed loading blocks the intended
consumer, report the exact call path and seek a source-cited amendment before
any parser change. No global positive-parser relaxation or error-string patch.

## Bounded discovery, only after approval

Use the admitted clearance control input with all four clearance values positive,
bridge ratios 1/1, identical geometry and all non-config ZIP member bytes.
Single-filament input; do not expand filament cardinality or geometry.
Three fixed inputs differing only in explicit `filament_flow_ratio`:
control [1], zero [0], upper admitted control [2]. Both positive controls require
complete ordered Native/API/CLI comparison with the existing generator-only
normalization and raw API=CLI. Do not compare different inputs to each other.
Record effective ratio and positive XY/E in accepted controls.

Native-first, then same-input validated API and real CLI even for Native rejects.
Record exact errors, exit codes, no-output status, input hashes, source pin,
canonical environment archive/hash and actual Native descendant/thread affinity.
CPU18, jobs2/threads2; serial official and seeded with separate fresh Native
caches/artifacts, required Native and all canonical references/oracle paths.
Expected zero diagnostic is `filament_flow_ratio: invalid value 0`, but live
Native serialization is authoritative. A different diagnostic requires review,
not assertion normalization or borrowed control G-code for rejected inputs.

All inputs equivalent: evidence-only NO-RED STOP; no passing-only test slice.
Native rejection versus Ares acceptance or earlier InvalidInput/CLI1: report
real divergence and exact ownership/call path for RED authorization. Never
fabricate a RED from environment, compilation, or witness faults.

## Included, deferred, and acceptance

Included discovery: scalar singleton vector zero admission and exact whole-vector
serialization rule at the cited validate boundary. Deferred: multi-filament and
mixed vectors, nullable/nil/NaN/negative values, profile cardinality/inheritance,
flow generation, `print_flow_ratio`, spiral validation, and every adjacent rule.
No new guards, clamping, fallback, independent pipeline, or runtime artifact commits.

After discovery approval only: frozen input-only fixtures and public test-first
RED, independent RED approval, then minimal GREEN consumer if source boundaries
permit. Check existing ares-core zero-rejection and zero-acceptance assertions
before GREEN review; preserve coverage at the owning layer. Preserve pure
processed lifecycle and accepted same-input raw control bytes RED to GREEN.
Targeted tests only as allocated; fmt/strict on exact clean final hash, changed
Rust files <=400 LOC. No full/workspace/merge/push; coordinator owns integration.
Honor active freezes and do not start Native before SPEC approval.
