# Machine-start overall chamber placeholder

Frozen Native-exported 3MF inputs; no regeneration or expected G-code archive.
Upstream GCode.cpp:3009-3012/3037 computes the used-extruder integer maximum
and registers ConfigOptionInt. The shared Rust base binding is already Integer;
only its machine-start overwrite is a Number compatibility shell.

| role | chamber_temperature | SHA256 |
|---|---|---|
| control | ["4"] | `f14985d07827445123e4e42ed68582e94c622bba651f12c472ab9ec7d87cfcc7` |
| target | ["5"] | `ddb1daa8c94b29eeaf41e5f92f33768a69465da99dad9abacf926f9f1b436e31` |

Both machine-start templates are exactly
`; CHAMBER {overall_chamber_temperature/2}`.
Only the chamber_temperature key differs in Metadata/project_settings.config;
all other ZIP member payloads are byte-identical. Both exported archives were
accepted by the official Native CLI. Dynamic Native references, not hardcoded
expected output, determine complete public stream parity.

Discovery files:
`/home/indexyz/ares-evidence/w6-overall-chamber-probe-discovery-4h7m2n9p/{control,target}.3mf`.
No new slicing or printer behavior is requested. The included custom comment
isolates the upstream derived ConfigOptionInt identity without masking the rest
of the generator stream. Formal public tests retain all M73/time/motion/comments/
metadata/footer output and run stable Native2 before API and CLI.
