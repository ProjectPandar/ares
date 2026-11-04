# Filament variant gather / initial retract

Source: pinned beta `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`PrintConfig.cpp:11862–11882,11971–11977`; nullable overlay
`Config.hpp:767–815`; retract consumer `GCodeWriter.cpp:1457–1503`.

`l59.3mf` is the unchanged nozzle-footer independent variant-column control
(SHA256 `908612c123f73537d6cd07b3baaf93e5c48e97f84503030944c330ac3327f23d`).
Its source columns are `[211,217,223,237]`, with self indices `[1,1,2,2]` and
Standard/High Flow variants. Active High Flow selects columns 1/3. Tool 1 must
have positive model extrusion. Original nullable length/speed overrides are
`[nil,nil]`; source column 3 is absent, so gather repeats the front nil.
Machine length/speed remain 4 mm / 60 mm/s: `G1 E-4 F3600` before the first
fan-layer marker.

The other archives change only `Metadata/project_settings.config`:

| File | Length override | Speed override | Initial retract |
| --- | --- | --- | --- |
| `nil.3mf` | `[nil]` | `[nil]` | `G1 E-4 F3600` |
| `zero.3mf` | `[0]` | `[nil]` | none |
| `nonzero.3mf` | `[2.5]` | `[37]` | `G1 E-2.5 F2220` |

All other ZIP entries and metadata are preserved. SHA256 identities are pinned
in the public tests. Tests require Native first/replay stability, positive tool-1
model activity, successful public API and CLI slicing, and raw API/CLI equality.
They compare the labelled first retract section exactly and retain separate
complete-output comparison artifacts. No general whole-output parity is claimed.

Failure modes: missing source column incorrectly becomes zero; nil becomes an
explicit override; explicit zero is incorrectly treated as nil; nonzero front is
not repeated; wrong variant column/tool selected; output slots reordered; Native
setup or replay fails; API/CLI differ. Empty-source early return is part of the
shared gather contract and must leave the destination unchanged, not fail or
create values. No private-helper unit tests are added.
