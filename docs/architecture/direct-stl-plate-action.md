# Plate selection after direct STL import and arrangement

Upstream: OrcaSlicer v2.5.0, `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

## Ordered model reads

`src/OrcaSlicer.cpp:1723–1729` checks existence inside the per-file loop and
exits on the first missing file (`CLI_FILE_NOTFOUND`, `-3`). `:1749–1758`
loads each STL with default instances. `src/libslic3r/Model.cpp:353–354,491–499`
routes STL to `load_stl`; `src/libslic3r/Format/STL.cpp:18–30` returns failure
for an unreadable or empty mesh. The read exception path at
`OrcaSlicer.cpp:1999–2002` reports the failing path and exits with
`CLI_DATA_FILE_ERROR` (`-6`). An empty STL follows the exception path, not the
empty-model continuation at `:2004–2007`. The codes are defined in
`src/libslic3r/Utils.hpp:45,48`; the report strings are at
`OrcaSlicer.cpp:197,200`. Both errors report plate index 0 and no sliced
plates. Files are checked and decoded in argument order before composing
profiles or arranging; a later missing path must not hide an earlier parse
failure. Reuse the existing STL reader and CLI error/report constructors.

## Slice action

`OrcaSlicer.cpp:2042–2045` resets a positive selector for non-Bambu imports
before the early plate-ID validation (`:5291–5298`) and arrangement. The
slice action re-reads the original argument at `:6441–6445`; the plate loop
at `:6522–6531` visits ascending plate IDs for `0` and only the matching
one-based ID for a positive selector. For fresh STL, a positive ID beyond
all arranged plates therefore selects nothing: success, no G-code, report
`plate_index` equal to the requested ID, no `sliced_plates` field, and
zero-valued slice settings. This differs from the early invalid-ID rejection
for a retained project.

Reuse the loaded-profile model preparation and CLI arrangement, then select
from the resulting project plates and run each through the existing
`Print` lifecycle. Retain one `plate_<id>.gcode` per sliced plate
(`OrcaSlicer.cpp:7183–7193`). Collect each completed plate's ID, triangle
count and warnings (`:6528–6531,7056–7063,7258–7273`) and write one Linux
report (`:535–583,8133–8136`). The direct STL CLI owns filesystem reads and
output naming; model assembly, placement and slicing stay in core.

The existing direct STL action already covers a missing single file, both
selectors on one plate, and multi-input arrangement that fits one plate.
The saved-project plate action covers selected 3MF input, not fresh STL.
This extension covers overflow into multiple arranged plates and errors
with multiple STL arguments. It does not change saved-project routing,
arrangement geometry, optional actions, or signed selector handling.

Verify with synthetic STL blocks and resolved stock presets against both
Native producers. Compare status, stderr, output inventory, every G-code
byte apart from the generator banner, and the entire parsed report apart
from unsigned wall-clock timings. Exercise all, plate 2, an out-of-range
positive ID, a malformed or empty later STL, and a missing later STL.
