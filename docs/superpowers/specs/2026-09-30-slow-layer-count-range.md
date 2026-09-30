# Slow-layer count range parity

Follow ARD-0064's ConfigOptionInt / PrintConfig / GCode speed-ramp boundary.
Replace small local samples with min 0, max 2147483647 and a reproducible
full-range interior. Also exercise 1 and 4 to distinguish disabled and active
short ramp behavior. No shrinking the range to model layer count.

Each request must survive original export, remain in final config and produce
fully matching ordered G-code. Preserve complete paired artifacts and show
active requests affect emitted motion, not only config comments. Exit requires
RED/GREEN, original direct tests and selected sweep, full Nextest, Clippy/fmt,
Tier-1 compiler checks, independent six-axis review and repair/re-review, then
one code+docs commit and push. This does not complete all option parity.
