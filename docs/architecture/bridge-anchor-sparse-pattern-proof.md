# Slice C1 SPEC/proof: exhaustive sparse-pattern admission

Status: proposed SPEC for direct deletion; no C1 production edit.
Base enum inventory: `c22f5a81da41fce715b528268711393649daad66`.
Independent slice from the proof-backed surface-kind deletion (Slice A).

## Proof

`crates/ares-core/src/options/process_options/object_source/enums.rs:157-217`
defines `ProcessInfillPattern` with exactly these 28 variants:

- Monotonic, MonotonicLine, Rectilinear, AlignedRectilinear
- ZigZag, CrossZag, LockedZag, Line
- Grid, Triangles, TriHexagon, Cubic
- AdaptiveCubic, QuarterCubic, SupportCubic, Lightning
- Honeycomb, ThreeDHoneycomb, LateralHoneycomb, LateralLattice
- CrossHatch, TpmsD, TpmsFk, Gyroid
- Concentric, HilbertCurve, ArchimedeanChords, OctagramSpiral

The first `matches!` in
`project_slice/prepare_infill/bridge_over_infill/transaction/anchor_projection.rs`
lists all 28 exactly once. Enumerating the enum and matching the guard's
`ProcessInfillPattern::` tokens gives equal sets, no missing/extra variant.
Hence negating this predicate is always false for every representable value
of the typed field `RegionOptions::sparse_infill_pattern`. This proof does not
depend on which public option schema accepts a particular role or pattern.
There is no legal enum discriminant that can reach this unsupported return.
Unknown enum text fails deserialization, rather than creating a 29th variant.

## Source boundary and requested change

Owning upstream: source `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`OrcaSlicer/src/libslic3r/PrintObject.cpp:2735-2754` and
`Fill/Fill.cpp:1394-1505` (`Layer::generate_sparse_infill_polylines_for_anchoring`).
The generator selects real grouped sparse fills and instantiates their Fill
implementation; it has no equivalent always-false admission rejection.
Reuse Ares' existing typed dispatch in `sparse_anchoring/arms.rs` and
`arms_tail.rs`. The upstream selected-surface logic remains authoritative.

Included: delete only the exhaustive sparse-pattern whitelist
and its unreachable unsupported return. Do not replace it with a default
pattern, new unsupported gate, error masking, or dispatcher rewrite.
`ProcessInfillPattern` remains imported for the separate top/solid checks.

Deferred: top/solid guards (Slice C2), density (B), float projection (D), all
Fill algorithms, rotations, geometry and future enum variants. A schema
invalid top/solid pattern is not an admitted sparse-pattern RED.

The existing validator is a temporary compatibility shell. Removing this
redundant list does not grow a separate Ares pipeline or change the owning
upstream dispatch boundary.

## Validation

Decide under the accepted unreachable-check policy:
direct deletion without fabricating a public RED. This proof is static,
not a passing/failed E2E result. No deletion is included in this document.

After the decision and RESTORED, run existing bridge/anchor and sparse-pattern
option_coverage regressions official+seeded with fresh per-run Native cache
and artifact roots. Strict clippy, fmt and wasm32 must pass.
C1 lands as its own commit. All assertions/comparators remain intact.

## Other slice audit still open

B: exactly zero bypasses anchor validation at the caller. Negative density
must be audited across effective region overrides as well as the global
loaded schema; a global 0..100 bound alone does not prove every region path.

C2: Native top/solid schema accepts eight patterns. The matching Rust helper
exists, but every public effective region path must be checked before claiming
these two checks unreachable. Do not conflate malformed input reaching a guard
with a Native-admitted RED.

D: f64 input finiteness does not establish finite f32 projected anchor length.
A schema-bounded percent near FLT_MAX, multiplied by spacing then narrowed,
may overflow; check Native admission and actual float projection after oracle
restoration. No unreachable-proof or deletion is claimed for D at this stage.

## Deletion decision / implementation

Decision `1e5ee06c3` accepts this exhaustive proof and permits direct deletion
of the sparse-pattern guard as a commit separate from Slice A. The production
change removes only that always-false rejection; the existing per-pattern
Fill dispatch and other checks are preserved. No fabricated RED or assertion
change is involved. Slice A and C1 may share one post-RESTORED directed run
while retaining distinct implementation commits.
