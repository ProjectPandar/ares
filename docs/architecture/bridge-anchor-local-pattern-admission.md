# SPEC C2: local top/solid pattern admission in bridge anchor preparation

Status: proposed SPEC, docs only; no input/test/production edit or run.
Base `96eb17ccc2805ce1d4f95045e452eb070e19f183`.
The A/C1/B
candidate `18d59a638` remains unchanged.

## Source boundary and audit finding

Owning source: `OrcaSlicer/src/libslic3r`, commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

| Source | Behavior |
| --- | --- |
| `PrintConfig.cpp:2074-2116,10400-10421` | Global top/solid option validation uses an eight-pattern allowed domain. |
| `Config.hpp:2084-2092` | Dynamic enum deserialization looks up the enum keys map, not the GUI option's allowed-value subset. |
| `PrintObject.cpp:3602-3659,3664-3709` | Copy local object/volume/material/range overrides to effective PrintRegionConfig; normalize density/filament/fuzzy fields, without a top/solid pattern whitelist. |
| `Fill/Fill.cpp:858-902` | Derive fill pattern from actual surface kind: top uses top pattern, internal-solid uses solid pattern, bridges select monotonic or rectilinear. |
| `Fill/Fill.cpp:934-941,1394-1505` | Role-specific rotation and real per-pattern sparse anchor generator. |
| `PrintObject.cpp:2735-2754` | Bridge preparation asks each required lower layer for its sparse anchors. |

Ares `options/region_options/metadata.rs:21-44` decodes
`ProcessInfillPattern` with the full enum. `overrides.rs:32-41` accepts known
region fields; `merge.rs:15-51` combines effective region overrides;
`normalization.rs:4-23` does not clamp top/solid patterns to eight values.
Thus a global schema argument does not prove the anchor top/solid checks
unreachable through local public metadata. Native admission is unproven.

## Rust destination and surgical split

Destination: existing
`crates/ares-core/src/project_slice/prepare_infill/bridge_over_infill/transaction/anchor_projection.rs`.
Use existing `group_fills/params/projection.rs` role selection and Fill dispatch.
The validator is a temporary compatibility shell, not a supported-input policy
independent of the owning upstream surface/Fill stage.

C2-T first: local `top_surface_pattern=grid`, otherwise normal sparse and solid
patterns. Native must admit this public input before removing its top-pattern
anchor gate. C2-S follows in its own RED/GREEN: local
`internal_solid_infill_pattern=grid` with valid top pattern. Do not combine both
production guard removals in one repair commit.

Included: only the selected role gate and proven missing behavior in its
existing Fill consumer, if a separate repair PLAN requires it.
Initial repair proposal is deletion of that selected gate only; no assumptions
that existing arbitrary-role Grid output already matches Native.

Deferred: other patterns beyond the bounded Grid witness, anchor floats (D),
new surface kinds, Fill algorithm redesign, rotations, density, materials,
layer-range slicing/height-profile work, brim, plate selection.
Other A/C1/B removals are not part of this new slice.

## Public Native admission preflight, then RED

After this SPEC: official and seeded runs, each with an independently
fresh Native cache/artifact root per invocation.

Use public 3MF exported from an existing vendor profile and a solid ceiling
fixture. Keep the project-level top/solid configuration valid. Insert one
known local object or model-part metadata key into `Metadata/model_settings.config`;
retain all other ZIP members' bytes/hashes, update the on-disk Native input to
exactly the same final bytes supplied to Ares. No internal record injection.

Cases per role slice:

- Active object override: Grid in the selected top/solid role; sparse density
  20%, a solid ceiling and actual Native internal bridge extrusion moves.
- Active model-part override: same single role, confirming the second public
  merge scope rather than assuming object metadata is the only route.
- Valid local control: rectilinear selected role on identical geometry.
- Inactive/no-candidate control: no internal bridge candidates by geometry;
  ensure ordinary selected-role output remains present where applicable.

Assert exact metadata key/value/scope and unchanged public input members.
Preflight Native must complete a nonempty full stream, including selected-role
extrusion moves. Compare active Grid role output with its rectilinear control
to prove configuration consumption (whole non-header bytes differ, and the
selected role's moves differ). Do not use a global config footer to claim
local override application.

For an admitted active case, the public Ares call must be observed rejecting
exactly `UnsupportedProjectFeature("top_surface_pattern")` or, in the second
slice, `"internal_solid_infill_pattern"`. The permanent expectation remains
whole-output generator-only PASS; the baseline fails that expectation. Keep
all raw streams, error, input/Native/Ares SHA, producer/launcher record and
CLI result in external artifacts before asserting. Submit RED as its own
commit. If Native rejects, or another Ares failure precedes this guard, stop
and report the actual result; it is not this slice's admitted RED.

If the active witness cannot be established, submit the admission/schema
finding for a decision, not a whitelist deletion justified by a passing control.

## GREEN and exit

After admitted RED: minimal selected-gate repair, preserving role/pattern/angle
selection, grouping, fill order, error propagation and write-back. If deletion
alone exposes a Fill mismatch, retain the RED and request a source-cited PLAN
amendment before expanding the production boundary.

Unchanged active/control assertions and whole-output API/CLI comparisons
must PASS both modes. Run selected bridge/anchor regressions, requested
option_coverage cases and strict clippy/fmt/wasm32. Files at most 400 LOC.
One commit per role slice; full parity before integration.
No new unsupported guard, silent error skipping, suppression, ignore, probe
short-circuit, regions[0], assertion weakening or committed Native outputs.
