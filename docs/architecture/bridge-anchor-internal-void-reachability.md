# Slice A: public InternalVoid witness cannot be established

Status: stopped before RED/GREEN; resolved by the deletion decision below.
SPEC: `d643736b0`.
Audited production base: `c22f5a81da41fce715b528268711393649daad66`.
No production edits, test edits, Native invocation or test run.

## Correction to the SPEC's fixture assumption

The KSR test in
`prepare_infill/bridge_over_infill/transaction/candidate_expansion/tests.rs:55`
filters surfaces for `InternalVoid` before retagging them. It does not assert
that this filtered set is nonempty. This is insufficient evidence that the
KSR predecessor contains voids. The SPEC's claim that it does was an incorrect
inference and is withdrawn. No admitted RED has been established.

All Rust paths below are relative to `crates/ares-core/src/project_slice/`.

## Exhaustive public-path proof

1. `project_slice.rs:181-201` (in the crate source directory) constructs
   `PreparedPostExternalSurfaces` by `external_surfaces::prepare`, then calls
   candidate preparation and bridge transaction preparation immediately.
   There is no intervening `clip_fill_surfaces` stage.
2. `prepare_infill/external_surfaces/stage.rs:45-78` processes every populated
   aligned input/record pair. Empty pairs contain no surfaces. A successful
   public slice cannot bypass `process_external_surfaces` for a populated
   record.
3. `prepare_infill/external_surfaces/process.rs:127-143` clears the entire
   surface vector and reconstructs it from five kinds:
   `InternalSolid`, `Internal`, `BottomBridge`, `Bottom`, `Top`.
   The appended zones explicitly use `InternalSolid` and `Internal`;
   bottom/top calls at lines 90-108 request those kinds;
   `external_surfaces/merge_bridges.rs:58` constructs `BottomBridge`.
   No output kind is `InternalVoid`, whatever kinds entered this stage.
4. `bridge_over_infill/stage.rs` gathers candidates from an immutable
   `PreparedPostExternalSurfaces` reference. Candidate collection does not
   mutate its predecessor's surface kinds.
5. `bridge_over_infill/transaction.rs:43-54` runs candidate expansion before
   surface rewrite or extra-bridge rewriting. Candidate expansion borrows
   the predecessor immutably. Lightning expansion stores separate surfaces
   in `BridgeCandidateObject`; it does not mutate the predecessor records
   supplied to the guard.
6. `transaction/candidate_expansion/layers.rs:101-106` passes precisely
   `record.fill_surfaces` from that predecessor to `anchor_projection::validate`.
7. `transaction/anchor_projection.rs:116-127` accepts all five possible kinds
   from step 3 (and also `InternalBridge`). `region_slices.rs:79-87` has only
   seven variants; `InternalVoid` is the sole excluded variant. It cannot
   arrive here through the current successful public preparation path.

Therefore the surface-kind UnsupportedProjectFeature return is unreachable
through the current public slicing path. Geometry minimization or public
configuration changes cannot produce the requested active rejection without
adding a new preceding surface-producing behavior or altering internal records.
Neither operation belongs to Slice A.

## Upstream distinction and boundary

Upstream commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`:
`OrcaSlicer/src/libslic3r/PrintObject.cpp:655-675` calls
`clip_fill_surfaces` between external-surface processing and bridge-over-infill.
`PrintObject.cpp:3863-3878` describes its void-preserving transformation and
activation; `:3943-3944` appends the difference as `stInternalVoid`.
`PrintObject.cpp:2833` includes void geometry in the deep sparse area.

Ares has no `clip_fill_surfaces` / `infill_only_where_needed` source symbol
in its current core. Porting this upstream stage is an adjacent milestone,
explicitly deferred by the SPEC. This report does not propose to
silently omit it, restore voids by retagging, or weaken the whole-output oracle.
The existing pre-external-surfaces creation of voids does not invalidate the
proof: step 3 removes those before the anchor guard.

## Requested decision

Classify this guard as proven unreachable on the current public path and decide
whether its deletion may proceed directly under the same proof policy given
for B/C/D, with ordinary public regressions after RESTORED. Alternatively,
retain it until a separate `clip_fill_surfaces` port creates a
public active witness. No production deletion is included in this report.

Native stable-options/dynamics restoration remains pending. No failed setup,
private record injection, unsupported return caused by another guard, or
already passing public case is labeled RED.

## Deletion decision and upstream static gate

Decision `f586ddc40` accepts direct deletion without a fabricated public RED.
The upstream source check resolves the earlier adjacent-stage suggestion:

- `PrintObjectSlice.cpp:22`: `PrintObject::infill_only_where_needed` is
  initialized to `false`. The source has no other assignment and no active
  public config entry for this static flag.
- `PrintObject.cpp:3871`: `clip_fill_surfaces` immediately returns when that
  same flag is false.
- `LayerRegion.cpp:954`: the other pre-bridge conversion to `stInternalVoid`
  also selects on this flag; false selects `stInternal`.

Thus upstream also produces no `stInternalVoid` before bridge-over-infill
through this public path. A port of the disabled clipping stage would be a
no-op here; it is not required to establish parity or justify this deletion.

The production change removes the surface-kind check and its
unused surface parameter/import. The caller keeps its aligned-slot invariant
but binds only the remaining input. All other checks, generation, rewrite
order and comparisons remain as before. No tests/assertions are removed.

Verification still requires the existing bridge/anchor public directed
regressions official+seeded after RESTORED, plus strict clippy, fmt and wasm32.
This direct deletion uses the accepted proof, not an E2E RED/GREEN claim.
