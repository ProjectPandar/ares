# Spec: Scalar-point probe wire format

Follow ARD-0051's ConfigOptionPoint/PrintConfig source boundaries and existing
Point2d Rust destination. Correct the four CLI authored probes to use the
upstream canonical scalar serialization, preserving their numeric values.

Acceptance: all four generated values survive original export unchanged;
both canonical and alternate input spellings export the specified coordinates;
wrong coordinates remain rejected; eight direct complete G-code comparisons
pass. Re-run the ordinary four-option sweep and retain proofs/full outputs.
Vector-point probe formatting and generic application checks remain unchanged.
These are representative points, not exhaustive numeric range coverage.
