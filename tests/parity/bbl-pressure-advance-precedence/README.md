# BBL pressure-advance precedence inputs

Source: `GCodeWriter.cpp:370-392::set_pressure_advance`: negative PA is omitted;
otherwise BBL command selection precedes flavor-specific selection. SPEC:
`docs/architecture/2026-10-24-bbl-pressure-advance-precedence-spec.md`.

Every input derives from the existing GUI-generated BBL one-object project,
retains BBL identity, disables the tower and sets `default_jerk=9`,
`travel_acceleration=6000`, and `pressure_advance=0.02`. Only saved flavor or
PA enable flag varies. No geometry, template, temperature or fan changes are
made. This is not a non-BBL stock-preset survey.

Native accepts BBL/Repetier, and that admitted boundary is in scope. No factory Bambu/Repetier hardware or firmware recommendation is
claimed; no writer state is manufactured.

| Input | Flavor / PA | SHA256 |
|---|---|---|
| bbl-stock-pa-control.3mf | marlin / enabled | 1d56a96dfa6bb2d86ef8a2d29be12ee0ea395e25d96172ee9a07a87f70456b14 |
| bbl-repetier-pa.3mf | repetier / enabled | f90235bc55fec8361f0ebe8ecf7b78239229e6dca766d7a647d794c55a295693 |
| bbl-repetier-pa-disabled-control.3mf | repetier / disabled | 92dbe0333f559c08a88f125018bf719dfcf2d2588b1cff98e7b059a634f58204 |

Three direct calls use the existing `native_stream_case` seam. Fresh Native
replay qualifies first. Callbacks read printer identity, PA enable flag and PA
value from the input, then inspect generated override records using the source
comment. Enabled cases must emit the input-derived M900 K value with BBL
L1000/M10 suffix. Disabled PA must produce no override. Unrelated template M900
commands are not mistaken for generated overrides. No hardcoded Native output
counts, coordinates, reference streams or Ares formatter calls are used. The
selected value has fewer than four significant digits, so ordinary input-derived
text suffices for this command-selection witness.

Complete Native/API/CLI comparison follows with the existing generator-line
normalization only. Timing, headers, configuration and footers remain included;
raw API==CLI is required. Callbacks are observations, not masks. Expected RED:
target fails at byte 66908, Native BBL M900 versus Ares Repetier M233; both
controls pass. Discovery's complete diff contains only that PA line, with equal
M73 records. Public runs must qualify independently. No Native outputs are
committed.

Failure modes were documented before production work: flavor-first precedence,
accidentally enabling disabled PA, formatter/negative-guard changes, adaptive or
reset state mutation, non-BBL command regressions, template-command confusion,
and hiding downstream output. Additional flavors, signed-zero and rounding
variants are outside this RED.

The port only moves the existing BBL match arm before flavor arms.
