# Explicit machine limits witness inputs

Synthetic W7 D3 inputs from the OrcaSlicer v2.5.0 discovery run at
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`. `source.json` identifies the source
BBL presets; the three settings files are the flattened stock preset inputs
from `worker7/parity-discovery/seeded/stock-inputs/10-BBL`.
`cube10.stl` is the repository's 10 mm cube placed on the bed in the positive
quadrant (W7 `direct-ground/10-BBL`). `a1mini.3mf` is the stock Native-exported
cube project from the same stock input directory, with A1 mini as the embedded
printer. `renamed.3mf` changes only `printer_settings_id` in
`Metadata/project_settings.config` to `D3 original printer name`.

Tests use these as inputs to both slicers; G-code is generated at runtime.
