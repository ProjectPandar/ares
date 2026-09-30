# Filename-format parity probes

Follow ARD-0063's PrintConfig / PrintBase explicit-file output boundary.
Replace baseline-only `filename_format` coverage with empty, literal, and
placeholder-bearing values. Each value must survive original export and
appear in final original G-code configuration; the complete ordered original
and Ares outputs must compare without new normalization. Retain artifacts.

Explicit CLI output naming is distinct from recommended filename evaluation;
do not claim the latter is covered. Exit requires RED/GREEN regression,
original direct and selected sweep evidence, full workspace Nextest,
Clippy/rustfmt and Tier-1 compiler checks, independent six-axis review and
repair/re-review, then one code+docs commit and push.
