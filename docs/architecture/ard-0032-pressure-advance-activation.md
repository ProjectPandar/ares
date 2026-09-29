# ARD-0032: Pressure advance coverage must activate emission

`GCode.cpp` gates `GCodeWriter::set_pressure_advance` on the filament-scoped
`enable_pressure_advance`. Set that enabler in the filament preset, not
the process or machine preset, and disable adaptive PA for this fixed-value
slice. Verify all three effective values through exported project bytes.

Use the existing schema-derived maximum/random generator; do not call a
hand-picked positive interior value a minimum. Preserve negative suppression
and zero reset as distinct behavioral probes. Whole-output comparison and
the incomplete-domain acceptance gate remain unchanged.
