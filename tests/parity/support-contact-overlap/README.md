# Normal support contact overlap

Each input contains one part built from `tests/parity/overhang.stl` at half
scale. `generate.py` reconstructs the mesh, plate metadata, and ZIP package
with fixed member timestamps. It reads only the resolved project settings
from OrcaSlicer v2.5.0 stock exports, checking their SHA-256 hashes, and then
applies the settings below. No Native G-code or other expected output is
stored. The upstream revision is `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

| Project | Stock presets (0.4 mm nozzle) | Settings under test | Contact layer |
| --- | --- | --- | --- |
| `x1c-grid-interface.3mf` | X1 Carbon / 0.20mm Standard @BBL X1C / Bambu PLA Basic @BBL X1C | `support_interface_pattern=grid` | Z=1 mm |
| `x1c-two-layer-raft.3mf` | same X1C presets | `raft_layers=2` | Z=1.6 mm |
| `k1-rectilinear-grid-base.3mf` | Creality K1 / 0.20mm Standard @Creality K1 (0.4 nozzle) / Generic PLA HF @Creality | `support_base_pattern=rectilinear-grid` | Z=1 mm |

All three enable Normal (auto) support with default style. The X1C projects
also disable `farthest_point_timelapse`. `generate.py` takes the resolved X1C
and K1 stock project archives in that order when run from the repository root.
The tests pin the resulting input hashes and compare the entire Native,
public API, and CLI streams with the generator-line-only comparator after
checking a second Native slice for determinism.
