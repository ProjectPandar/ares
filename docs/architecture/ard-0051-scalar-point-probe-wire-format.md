# ARD-0051: Use ConfigOptionPoint's canonical wire format for probes

`Config.hpp:1452-1485::ConfigOptionPoint` accepts comma or `x` between
coordinates but serializes a scalar point with a comma. This differs from
`ConfigOptionPoints` (1487 onward), whose vector elements serialize as `XxY`.
`PrintConfig.cpp:2250-2280,3818-3822` owns the three bed-mesh scalar points
and `best_object_pos`. Existing Rust destination:
`ares-core::options::config_types::point::Point2d` and the corresponding
typed printer settings; the CLI domain probes are a temporary validation
shell around these source-owned configuration concepts.

The four probes currently request the accepted alternate `x` spelling,
then incorrectly require that spelling to survive canonical export. The
coordinates are unchanged. Generate their canonical comma spelling instead:
retain `(5,5)` and `(0.25,0.75)`, the same case counts and existing context.
Do not normalize arbitrary strings or relax requested/effective checking.
Keep polygon/vector-point probes in their separate `XxY` representation.

Verify all four generated scalar probes through actual original export and
complete G-code comparison, also exporting each former `x` spelling and
checking the same independently specified canonical coordinates. A changed
coordinate must still fail the application check. The alternate-spelling
tests cover the original export boundary, not raw alternate-spelling 3MF
deserialization by Ares. No production option parser or slicing code changes.

Included: representative scalar-point export, config output and full-output
equality on the existing Ender-3 cube. Deferred: expanded coordinate ranges,
adaptive-mesh template activation, layout combinations and other point types.
These probes do not establish exhaustive geometric coverage or global parity.
