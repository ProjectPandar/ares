# Plan: Task 22o.117 layer metadata float semantics

1. Add a failing KSR assertion for exact float-derived layer metadata and rejection of binary-float tails.
2. Port Orca's float layer-Z cache and six-significant-digit processor formatting while replacing the quadratic cumulative-height scan.
3. Carry the source float-difference height into planned extrusion flows; update behavioral expectations and remove the obsolete raw-bit pinning assertion.
4. Run profile-layer and focused output tests, smoke-slice the KSR 3MF, then run rustfmt and clippy before committing and pushing.
