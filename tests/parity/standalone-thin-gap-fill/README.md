# Standalone thin-gap-fill tool collection

Synthetic geometry: a 20 mm square frame with a 1.2 mm band and a
16 mm solid block, both 2.4 mm high. The manifold mesh has 24 vertices
and 44 triangles. The frame produces standalone GapFill before the
block produces genuine sparse infill.

The existing cube-control archive supplies the ordinary Ender-3 profile
and package structure, not the model geometry. Two loaded PLA filaments
share one nozzle. Walls and solid surfaces use filament 1; sparse infill
uses filament 2. Stock startup, shutdown, timers and M73 remain enabled.
The four-entry flush matrix retains the template's diagonal and cross-slot
values; profile bookkeeping is resized for two filament slots.

Regenerate with:

```sh
python3 tests/parity/standalone-thin-gap-fill/generate.py
```

`manifest.json` pins the template, generated archive and every ZIP payload.
The test uses the existing `NativeCase` Native/API/CLI complete-stream
comparison. Native GapFill role markers must use tool 0 and sparse role
markers tool 1, with GapFill preceding the first tool-1 switch.
No comparator or normalization change is part of this case.

Owning source: `Fill.cpp:1528–1537` independent thin-fill wrappers and
`ToolOrdering.cpp:144–164` collection-to-filament selection in OrcaSlicer
2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`).
