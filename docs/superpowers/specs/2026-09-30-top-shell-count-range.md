# Top-shell count range parity

Follow ARD-0066's ConfigOptionInt / PrintObject top projection boundary.
Generate min 0, max INT_MAX and a fixed-seed full-range interior; retain each
exact original exported request, final config and complete paired output.
Preserve the original's observed 32-bit count-end result with explicit Rust
arithmetic; also compare INT_MAX-1 and INT_MAX-100 boundary controls.

Exit requires actual matching complete outputs, RED/GREEN regression, selected
sweep, full checks, independent six-axis review/repair, then a code+docs commit
and push. A documented discrepancy is diagnostic progress, not completion or
proof that unrelated undefined behavior should be reproduced. Keep failures
visible until fixed.
