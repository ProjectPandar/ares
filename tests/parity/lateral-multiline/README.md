# Lateral multiline overlap RED inputs

These ten 3MF files are input-only fixtures frozen for the bounded RED
covering the upstream `FillRectilinear.cpp:2960-2994` `remove_overlapped`
helper and its `FillRectilinear.cpp:3031-3032` lateral-pattern/multiline gate.
Native G-code is generated at runtime by `OrcaRunner`; no Native output or
receipt is stored here. `SHA256SUMS` pins the authored input bytes.

The six candidate inputs are LateralLattice/LateralHoneycomb multiline 2/3
cases; the four controls are count-1 lateral cases plus Cubic and
QuarterCubic multiline-2 controls. Existing support-ironing assertions remain
untouched.
