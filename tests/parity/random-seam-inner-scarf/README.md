# Random seam arithmetic exposed by inner-wall scarf

These are the actual official OrcaSlicer 2.4.2 CLI-exported projects,
not post-export edited ZIPs. `manifest.json` freezes their complete
hashes, every ZIP payload hash, and retained GUI settings.

The ZIP payloads differ only in `Metadata/project_settings.config`;
the JSON differs only in `seam_slope_inner_walls`: `"0"` for control,
`"1"` for positive. Geometry and all other settings are identical.
Scarf Contour, random seam, staggered inner seams, three walls and
Inner/Outer order are retained. Wipe is disabled using the legitimate
per-extruder `wipe = ["0"]` setting, avoiding two separate retraction
mismatches rather than filtering output.

The source-cited boundary and read-only GDB evidence are in
[the SPEC](../../../docs/architecture/2026-10-20-random-seam-inner-scarf-spec.md).
The last-edge guard remains unchanged per the spec amendment.
This witness depends on the separate staggered interpolation
fix; it is NOT the previous slice_z12.5 staggered witness.

## Formal contract

Tests directly reuse
`runner/native_api_cli.rs::NativeCase`, without copying either helper,
manual Native/API/CLI flow or the borrowed PROFILE_ENTRY gate. Native
availability is the only environment condition, like main's NativeCase
users. Targeted filter: `test(random_scarf_normalized_edge_)`.
The current source boundary is re-established as SPEC5047755ca.


Fresh official and seed-replay Native outputs are compared against the
public validated slicing API and bare Ares CLI, using the shared
Native/API/CLI harness. Compare complete streams, permitting only the
existing generator identity/date-line removal. Retain Native/API/CLI
G-code, CLI logs and the comparator result as external artifacts.

Before the random arithmetic fix, control must PASS and positive must
FAIL on precisely one ordered output line (raw line 20461, or 20460
with the generator line removed):

    Native: G1 X114.679 Y118.918 Z18.329 E.02273
    Ares:   G1 X114.679 Y118.918 Z18.329 E.02274

The positive pins Native's complete witness line as a premise. After
the bounded fix, require both COMPLETE outputs to pass in both modes;
any residual mismatch blocks scope expansion. The isolated local probe's
read-only GDB record matched all100 random selections and the target final
Y f32 bits3225649160. That proof remains local probe evidence, not a
current-main integration claim.

Native export/slice reachability can be independently repeated with
an official Orca CLI (`$ORCA`), writing outside the repository:

    "$ORCA" --export-3mf "$OUT/scarf-positive-exported.3mf" positive.3mf
    "$ORCA" --slice 0 --outputdir "$OUT/scarf-positive" positive.3mf

Run these commands from this fixture directory with `OUT` set to an
external directory, using distinct output locations for control.
