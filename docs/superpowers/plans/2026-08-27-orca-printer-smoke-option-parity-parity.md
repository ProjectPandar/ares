
## 2026-09-05 (cont 271): KSM 0.4 = E±1e-5 + grid family swap (different spacing)

G-code-only diff (movement lines): first divergence at line 221 =
E.37995 vs .37996 (extrusion rounding ±1e-5); second at 228-230 =
GRID FAMILY SWAP — ref walks vertical family (y:171.374→173.002),
mine walks horizontal family (x:171.374→173.002). Same 45° grid
but the PERPENDICULAR family at the same index. The sweep order
fix (1768c2f3) fixed the 0.6 case (width 0.62, spacing 8.27mm,
1 line/family) but NOT the 0.4 case (width 0.45, spacing 6.0mm,
multiple lines/family). With different spacing, the grid anchor
(first_x = align_to_grid) lands differently — the family mapping
flips. ROOT: the sweep-to-family correspondence depends on the
spacing-to-region size ratio; the fixed SWEEPS order [pi/2,0] maps
differently at different spacings. NEXT: (1) verify which sweep
angle produces which family for the 0.4 spacing (dump GRID angle +
the first vline position for both cases); (2) make the family
mapping spacing-independent (likely the extra pi/2 belongs in the
per-sweep total rather than the base angle — the fix from cont 228
needs refinement); (3) separately trace the E±1e-5 rounding path.
