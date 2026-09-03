
## 2026-09-03 (cont 63): SPARKX i7 0.2 nozzle — sub-micron family

The filament-length cluster (15 printers) reproduces on the Creality
SPARKX i7 0.2 nozzle at /tmp/sparkx. Divergence: 223.03 vs 222.07mm
(~1mm). Breakdown:
- 510× F1605 vs F1616/F1615 — cooling buffer feedrates ±10-11 mm/min, cascading from the ~1mm volume difference
- 98× path position diffs at ~60μm (X134.313 vs X134.252) — polygon vertex rounding cascade
- 1× G2 Z3 vs G2 Z13 spiral lift at a specific late layer — a Z-level divergence in the end-game
Root cause: sub-micron vertex rounding (the same Clipper precision family as ksr layer-4). The ~1mm filament total difference is the accumulation of the vertex rounding across 50 layers.
