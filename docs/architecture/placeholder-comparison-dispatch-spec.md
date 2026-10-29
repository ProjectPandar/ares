# Placeholder comparison dispatch rewrite

## Upstream boundary and Rust destination

Port OrcaSlicer `src/libslic3r/PlaceholderParser.cpp:558-600`,
`expr::compare_op` and its derived comparator wrappers, as one dispatch unit
into `crates/ares-core/src/project_slice/gcode_emit/expression/comparison.rs`,
called by expression.rs::parse_compare. Preserve the existing regex branch.
The existing permissive as_number()/as_string() fallback is a temporary
compatibility shell and is replaced only for these comparisons.

## Required dispatch

- Both numeric: compare integers directly when both TYPE_INT, otherwise double.
  Double equality uses abs(a-b) < 1e-8.
- Both boolean: equality only (inequality derives by inversion).
- Either string: compare to_string() values lexically for equality/less/greater.
- Other type combinations: error.
- Derive !=, <=, >= by inversion of equality, greater, less respectively, as
  upstream does (not independent floating operators).

Upstream `expr::to_string` :302-327 uses std::to_string for integers,
true/false for booleans, and default ostringstream precision for doubles.
Reuse existing six-significant-digit ostream formatter in
`gcode_emit/motion/format.rs::ostream` for comparison double conversion,
widening only its internal visibility if needed. No global Value coercion
or formatting changes. Non-numeric type error wording follows compare_op.

## Exclusions and verification

No lexer/grammar, arithmetic, regex, function behavior or global as_number
changes. Do not grow a new pipeline. Tests use official Native-exported
GUI machine_start_gcode comments and shared strict stream comparator:
control `{"2" == 2}`, positive `{"02" == 2}`. One additional positive
`{"1.23457" == 1.23456789}` has actual observed Native true / Ares false
and exercises the same string dispatch plus upstream double to_string.
Both positives must genuinely fail before implementation, then all pass in
both official/seeded modes. Existing generated-by normalization only.

Discovery receipts and inputs:
`/home/indexyz/ares-coord/evidence/w5-a887-string-compare-discovery/` and
`/home/indexyz/ares-coord/evidence/w5-a887-string-number-discovery/`.
Stack on approved a8870d1f6. No SPEC commit implementation or test changes.
