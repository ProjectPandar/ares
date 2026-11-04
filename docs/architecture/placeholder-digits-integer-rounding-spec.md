# Placeholder digits without decimals: integer rounding

## Upstream rewrite boundary

Port only the no-decimals `digits` branch from OrcaSlicer
`src/libslic3r/PlaceholderParser.cpp:670-684` into
`crates/ares-core/src/project_slice/gcode_emit/expression/functions.rs`,
`function`'s `digits` no-decimals formatting branch.

Upstream `:683` formats an integer (`%*d`) obtained via `as_i_rounded`.
At `:275`, TYPE_INT is passed through, while TYPE_DOUBLE is converted with
`int(std::round(d))`: half ties round away from zero before integer formatting.
Ares currently formats f64 directly with zero decimals, yielding ties-to-even.
That formatting is a temporary compatibility shell, not the upstream behavior.

## Non-negotiable included behavior

Only two-argument `digits(value, width)` changes. Preserve TYPE_INT directly;
for a floating-point argument round half away from zero, cast to upstream's
integer representation, then format with the existing space-padding width.
No new dependencies, fallback, parser/value-model redesign or output constants.

## Deferred behavior

Do not change the three-argument decimal formatting branch, leading-zero
formatting (`zdigits`), other functions, argument admission/error messages,
locale behavior or out-of-range C++ conversions.

## E2E acceptance

GUI Machine start G-code input comment `{digits(2.4, 1)}` is the control;
`{digits(2.5, 1)}` is the positive. Both are official Native-exported 3MFs.
The shared Native stream comparator must expose the positive mismatch before
implementation and pass both afterward in official and seeded modes. Preserve
whole-stream comparison and existing generated-by normalization only.
