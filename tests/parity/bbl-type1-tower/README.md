# Stock BBL Type1 tower public parity witnesses

Source boundary: Print.hpp:1086; Print.cpp:3290-3301,3390-3541;
GCode/WipeTower.cpp::generate_new/plan_tower_new; GCode.cpp::append_tcr and
first-layer bounds. See the Type1 milestone ARD and T1–T2 consumed-chain SPEC.

`enabled.3mf` is the accepted official-exported discovery input, copied byte
for byte: stock X1 Carbon 0.4 / 0.20mm Standard, two cube10 objects assigned
Bambu PLA Basic and Bambu PLA Matte in two AMS slots on the same nozzle.
Original discovery exported two objects from tests/parity/cube10.stl, selected
stock presets and materialized two-slot project vectors before official Native
re-export. No machine or filament template injection. Provenance/scripts and
raw discovery are retained externally.

`disabled.3mf` changes only the GUI Prime tower → Enable setting:
Metadata/project_settings.config enable_prime_tower "1" → "0". JSON formatting
changes; all other decoded settings and every other archive member are identical.
The original enabled input is never regenerated or normalized. Input SHA256:

- enabled: ab00b9040c3ccc1c10490a220880b87108aed7edd3a19926b8e7967f1287e506
- disabled: e78d0feba53df6318deb96929eb0620d5392f16a88832fb4ffd75969b853ed28

Both cases require successful, repeat-stable Native output before public API
and actual CLI whole-stream comparisons. The sole comparison normalization is
the existing generated-by line handling. No expected Native output is stored.
A disabled control is not assumed to pass: any residual full-stream failure
must be reported, never masked or changed into expected failure.
