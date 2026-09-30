# Bottom-shell count range parity

Follow ARD-0065's PrintConfig scalar count and PrintObject shell projection
boundaries. Replace local samples with 0, INT_MAX and a reproducible full-range
interior. Each request must survive original export, remain in final config,
and produce matching complete ordered G-code with retained paired artifacts.
Verify the fixture has zero bottom-shell thickness and that active counts
change emitted motion versus zero.

Exit requires RED/GREEN, direct original-binary tests and selected sweep,
full Nextest, Clippy/rustfmt, Tier-1 compiler checks, independent six-axis
review and repair/re-review, followed by one code+docs commit and push.
Other shell/count combinations and global parity remain outstanding.
