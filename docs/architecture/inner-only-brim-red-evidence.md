# Inner-only brim test-first evidence

The RED commit changes tests/input fixtures and documents only; the production
InnerOnly dispatch is still absent at its parent.

The input-only generator and manifest are in
[`tests/parity/inner-only-brim`](../../tests/parity/inner-only-brim/README.md).
The E2E test lives in `ares-cli/tests/orca_parity/inner_only_brim.rs`, registered
under the existing brim test module. Every run uses canonical control-set-v2
and fresh Native cache/artifact roots per mode.

## Observed RED

Final test sources were rerun in both modes. Each mode executes
15 tests: the nine new inner-only cases and the original six N/O/E controls.
Six active cases fail the unchanged complete ordered-byte comparator; three
inactive controls and all six original controls pass. Exit100 is test RED,
not a Native/admission/build/resource failure.

| Case | Native positive brim XY/E motions | Ares motions before implementation |
|---|---:|---:|
| rectangle | 223 | 0 |
| concave | 313 | 0 |
| two-holes | 99 | 0 |
| narrow-neck | 158 | 0 |
| positive-gap | 227 | 0 |
| collapsed-inset | 180 | 0 |
| zero-width / sub-pair-width / no-hole | 0 each | 0 each |

Both modes admit the input in Native (result return_code0), the validated
public byte API and the real CLI (exit0). Native effective footer settings
confirm inner_only, requested width/gap and brim_use_efc_outline0. All positive
Native brim extrusion witnesses are at first-layer z0.2 and have endpoints
and midpoint inside the input holes. The first full byte difference is at
byte847, line32, the M73 progress estimate; the missing brim motion and changed
extrusion/time footer are retained in the complete streams. M73 is evidence
of the missing path's whole-output effect, not a separate normalization target.

API admission is recorded separately from comparison. No header/footer,
assertion or comparison normalization is weakened to create RED.

## Checks and implementation boundary

The generator reproduces all nine input archives byte-for-byte and verifies
closed directed mesh edges and positive volume. Archives contain no G-code.
No production file or public adapter assertion changes.

Implementation follows `Brim.cpp:450,547-558`: use the original paired
quantization of brim_width, never outer brim_width_mod. Hole offsets use
upstream default Miter/limit3 without SCALED_RESOLUTION; objectIsland uses
Round. Preserve no-outer masks, final bed subtraction and contact filtering,
then reuse the shared makeBrimInfillImpl paths and existing erBrim emission.
