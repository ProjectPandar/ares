# Fixed pressure-advance signed-zero public RED

Source `GCodeWriter.cpp:370-391`, startup consumer `GCode.cpp:3155-3161`.
SPEC `docs/architecture/2026-10-24-pa-signed-zero-spec.md`.
Test-only stack atop approved frozen jerk `4c24dd6d5`, including frozen lift
`2692155a9`; no production changes. Temperature `a5cc569c0` remains separately
frozen until the combined integration window.

All archives derive from stock `orca_cli_ender3.3mf`, tower disabled. The positive
and negative cases enable fixed PA and differ ONLY by pressure_advance0 vs-0.
Stock control leaves fixed PA disabled. Adaptive PA, templates, geometry,
temperature, jerk and normal0.4 lift remain stock.

| Input | SHA256 |
|---|---|
| stock-control.3mf | a360b2d4d71a140a197726b8ab4c8c8b370ad2e17957fa742da2a2346f9336ae |
| positive-zero.3mf | 67ffaf0ffe1409ebf2a264fe3b8b087b84ca61cc035db12a529be3077e55553b |
| negative-zero.3mf | 752f7ddf2c430856d90877a5e902100338fb9689ce62884cf7c9bec27c7e6fab |

Real filament options: `PrintConfig.cpp:2340-2351`, `Tab.cpp:3936-3941`.
Saved floating signed-zero values are accepted by Native. GUI display/sign
normalization is not asserted. This is byte fidelity only, not different
physical PA behavior or a printer safety claim.

Native-first `native_stream_case::check` qualifies fresh reference+replay, then
the callback parses the actual Native PA command argument as f64 and checks its
IEEE zero classification and sign bit. Disabled control requires no PA override.
No checked-in Native outputs or output-coordinate/count constants. Entire public
API+CLI streams, including configuration and timing, are compared through the
unchanged generator-only comparator; no masking or normalization of zero signs.

Expected RED: only negative-zero diverges at the emitted minus sign; positive
and disabled controls remain full byte-equal in both modes. Failure classes
before implementation: zero sign collapse, excluding negative zero as strictly
negative, changing positive zero or disabled behavior, changing nonzero precision
or parsing as a workaround. The eventual authorized GREEN is limited to the
private formatter's sign-preserving zero branch; none accompanies this RED.

Grouped registration appends an ordinary separated block in
`object_emission_cases.rs`; root and runner stay unchanged without squeezing.
