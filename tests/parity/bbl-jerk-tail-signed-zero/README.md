# BBL jerk tail signed-zero inputs

Source: `GCodeWriter.cpp:58-59` copies raw Z/E jerk caches; `298-299` streams
them with precision two and retains negative zero. SPEC:
`docs/architecture/2026-10-24-bbl-jerk-tail-signed-zero-spec.md`.

Every input derives from the existing GUI-generated BBL one-object archive,
keeps stock marlin flavor and BBL identity, disables the tower, and sets
`default_jerk=9` and `travel_acceleration=6000`. Only the saved machine Z jerk
changes for the target and zero control. Geometry, templates, PA, temperature,
fans and all other process settings remain unchanged. These are Native-admitted
saved floats, not physical tuning or GUI decimal-display claims.

| Input | Z jerk | SHA256 |
|---|---|---|
| stock-control.3mf | Stock 3 | 6cb522350420360807f04e3e920a71c71286a92583f2d513a8def2eec5b2c975 |
| negative-zero-z.3mf | -0 | c9eb20cb0b3390c71180e76854c80c9f81357e24e40abfba652f6582584f4891 |
| positive-zero-z-control.3mf | 0 | aa4f1d49d031216cc1fc3c325bc9562f0c3527e7499ac687336e1a6ce8421ce7 |

Three direct calls use the existing `native_stream_case` seam. Fresh Native
replay qualifies first. Callbacks read saved Z values and inspect actual body
M205 Z fields, excluding the raw envelope. Numeric values must match; zero
classification and sign bits are compared separately. Numeric equality alone
cannot distinguish negative and positive zero. No Ares formatter calls,
hardcoded Native coordinate/count goldens or Native outputs are committed.

Complete Native/API/CLI comparison follows with existing generator-line
normalization only. Timing, headers, configuration and footers remain included;
raw API==CLI is required. Callbacks are observations, not masks. Expected RED:
target fails at byte 67330, Native Z-0 versus Ares Z0; both controls pass.
Discovery's entire target diff contains only 299 body Z signs, with equal M73
records. Public runs must qualify independently.

Failure modes were written before implementation: erasing negative zero,
changing positive zero or nonzero/default-six formatting, confusing equality
with sign equality, fixing one caller instead of the shared source boundary,
or modifying unrelated axis/extrusion/zero-E/configuration paths.

Freeze these tests and archives before GREEN. The later change only preserves
sign in the existing shared ostream zero branch. Because all callers share it,
GREEN must run every caller's existing directed neighbors in both modes,
including jerk, acceleration, velocity limit, square corner, ACCEL_TO_DECEL,
Repetier, Klipper, expression/placeholder and coordinate-label consumers, plus
fixed PA signed-zero cases. See the SPEC's inventory and STOP contract. Any new
residual requires an immediate report, not wider edits or masks. No new public
default-six signed-zero witness is claimed by this three-case RED, and no
non-BBL stock survey is included.
