# Spec: Enum application evidence filenames

Use ARD-0050's source-owned `WallSequence` boundary and existing Rust
destination. Repair only the CLI application-evidence filename projection.

Acceptance: all three declared wall-order values are exported unchanged,
each has a separate readable proof JSON directly under the artifact root,
and every case reaches strict complete G-code comparison and passes.
Repeating a filtered case must update its own proof without overwriting a
different case or creating another identity. Domain completion rules and
generator-only comparison allowances remain unchanged.

Validate through the established option-sweep/project-to-output seam,
not source-token pinning. Cover generic and typed-width proof persistence;
do not change production slicing or claim global parity completion.
