# SPEC: bridge-over-infill anchor admission alignment

Status: proposed SPEC; no implementation or RED run yet.
Base commit: `c22f5a81da41fce715b528268711393649daad66`.

## Owning upstream boundary

Port `PrintObject::bridge_over_infill` and
`Layer::generate_sparse_infill_polylines_for_anchoring` / `group_fills`;
this is not a new Ares slicing stage. References below are relative to
`OrcaSlicer/src/libslic3r`, source commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

| Source | Behavior |
| --- | --- |
| `PrintObject.cpp:2468-2589` | Gather internal-solid bridge candidates using all lower regions. |
| `PrintObject.cpp:2735-2754` | Generate anchors on the lower layers required by actual candidates. |
| `PrintObject.cpp:2823-2842` | Deep sparse area includes `stInternalVoid`, independently of sparse density. |
| `Fill/Fill.cpp:858-902` | Group by actual surface kind and role; voids are recorded separately; nonpositive sparse density produces no sparse fill. |
| `Fill/Fill.cpp:934-941` | Apply sparse/solid rotation templates to their owning roles. |
| `Fill/Fill.cpp:978-989` | Project absolute/percent anchor lengths through float and clamp anchor to maximum. |
| `Fill/Fill.cpp:1394-1505` | Generate anchors from grouped `stInternal` fills, using each pattern's real generator; other surface roles do not become sparse anchors. |
| `PrintConfig.cpp:2074-2116,2969-2978,3137-3187` | Top/solid pattern domains, density 0..100, absolute/percent anchor schema. |
| `PrintConfig.cpp:10400-10421` | Native validates the allowed option enum domains. |

Rust boundary:
`crates/ares-core/src/project_slice/prepare_infill/bridge_over_infill/transaction/anchor_projection.rs`,
its call in `transaction/candidate_expansion/layers.rs`, and only proven missing
behavior in existing `sparse_anchoring` / `group_fills` consumers. Existing
transaction admission is a temporary compatibility shell; it must not define
an independent supported-input subset of the cited upstream stage.

## Inventory on the assigned base

The requested historical list differs from the current file. There are seven
`unsupported(...)` return sites, but rotate-template and extruder-order guards
have already been removed by prior merged slices. Preserve their behavior.

| Current check | Reachability / intended slice |
| --- | --- |
| `sparse_infill_pattern` | Whitelist covers all 28 current `ProcessInfillPattern` variants; redundant for typed inputs. Slice C. |
| `sparse_infill_density <= 0` | Caller already bypasses validation for exactly zero; schema rejects negative density. Slice B. |
| `top_surface_pattern` | Whitelist mirrors Native's eight allowed values; stage-local rejection must not replace public schema validation. Slice C. |
| `internal_solid_infill_pattern` | Same eight allowed values as top pattern. Slice C. |
| `!anchor_max.is_finite()` | Float projection may differ from finite public input; audit exact upstream float conversion and public schema first. Slice D. |
| `!anchor.is_finite()` | Present although not named in assignment; same projection audit as maximum. Slice D, report before changing. |
| `bridge_over_infill_anchor_surface_kind` | Rejects the one current variant absent from its whitelist: `InternalVoid`. Slice A. |

Do not fabricate an admitted public RED for a provably unreachable check.
Slices B/C/D need their own evidence and SPEC; the inventory alone
is not permission to remove checks without the requested TDD evidence.

## Slice A proposed first: InternalVoid anchor context

Included:

- Let naturally produced `InternalVoid` surfaces coexist with nonempty
  internal-bridge candidates and lower-layer sparse-anchor generation.
- Preserve upstream role selection: voids produce no sparse extrusion, and
  only grouped `Internal` surfaces feed the anchor generator. This is the
  explicit upstream surface selection, not a workaround that discards regions.
- Retain void geometry in the existing deep-sparse-area consumer, corresponding
  to `PrintObject.cpp:2833`; no retagging voids as sparse infill.
- Remove the transaction's surface-kind unsupported gate once the public RED
  proves its mismatch and existing consumers match this source behavior.
- If its surface argument becomes unused, remove that argument and its import
  from the single caller; no unrelated transaction refactor.

Deferred:

- Density, pattern-schema and float-projection slices B/C/D.
- New surface variants (including upstream `stSecondInternalBridge`) and a
  new `clip_fill_surfaces` implementation; these require separate source slices.
- Lightning/adaptive/Stars algorithm changes, fill rotation, brim, plate
  selection and G-code behavior.

Existing `group_fills/params.rs:110` observes voids separately and existing
`sparse_anchoring/mod.rs:73` selects only `Internal`. Use these existing ports;
do not introduce a second fill dispatcher or a new fallback.

### Public E2E RED contract (before production changes)

Use the public slicing API and the existing Native-backed CLI comparison
harness under `crates/ares-cli/tests/orca_parity/`. Build fixtures using public
3MF/config inputs only; never inject or retag internal Rust surface records.

1. Minimize a stepped/overhanging solid with sparse lower-layer fill and an
   internal solid ceiling. The existing real KSR bridge-candidate fixture is
   a diagnostic starting point: its predecessor contains `InternalVoid`.
   This does not establish a public E2E witness by itself.
2. Active case must naturally reach the exact surface-kind rejection while
   Native successfully emits a nonempty slice and internal bridge moves.
3. Controls: a matching case without the void-producing geometry/configuration,
   and a no-candidate case. Both must complete and preserve whole-output parity.
4. Assert Native success, exact public project/config application, nonempty
   active bridge witness, and existing whole-output API/CLI comparison PASS.
   On baseline the active PASS assertion must fail because Ares returns
   `UnsupportedProjectFeature("bridge_over_infill_anchor_surface_kind")`.
5. Save repeatable input, raw Ares/Native SHA, API/CLI output identity, witness
   and comparison record in the run's fresh artifact root. Native outputs
   stay outside Git. The same input must be used for Ares and Native.

If the active public witness cannot be established, stop and report that
fact before production edits. If removal exposes another missing upstream
behavior, report it for a scoped PLAN amendment; do not bypass comparisons.

Run official and seeded with fresh Native cache/artifact roots independently.
The canonical stable-options/dynamics oracles must be restored before any
dependent directed test or full parity run.
No test has been run for this SPEC.

Commit admitted RED separately and report its hash before GREEN. Keep all RED
assertions/comparisons unchanged in GREEN. Add no UnsupportedProjectFeature
replacement, silent error handling, guard fallback, suppression or ignore.

### Slice A GREEN exit

- Active and controls PASS official + seeded.
- Whole option_coverage official + seeded; changed files at most 400 LOC.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- `cargo fmt --all --check`.
- `cargo check --target wasm32-unknown-unknown -p ares-core -p ares-vgcode -p ares-wasm`.
- Full parity run, because this changes bridge-over-infill.

## Follow-on order (not bundled into Slice A)

B: zero-density semantics and redundant caller admission. Verify zero/no-anchor
and mixed-region cases, preserving Native's solid-surface work at density zero.
Do not invent sparse paths or bypass the whole region when density is zero.

C: exhaustive typed pattern dispatch and public option-domain validation.
First document whether top/solid out-of-domain values are rejected at the
public boundary. No permissive Native-divergent pattern interpretation.
Rotation/extruder controls remain regressions of their already merged ports.

D: anchor/maximum float projection, with ordinary absolute/percent and finite
public overflow probes. Native must admit the failing domain to qualify as
public RED. NaN/Inf text rejection is not an admitted RED. If only malformed
input or unreachable states can trigger a check, submit that result for an
explicit decision rather than weakening TDD or labeling it GREEN.

Each follow-on gets a separate SPEC, RED and commit; no combined
seven-guard production commit.
