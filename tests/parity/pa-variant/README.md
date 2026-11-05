# Pressure-advance filament variant cohort

Source: pinned beta `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`PrintConfig.cpp:10199–10205,11862–11882,11971–11977`.
Static PA consumer: `GCode.cpp:11330–11340`, `GCodeWriter.cpp:599–637`.

Tests reuse unchanged `../variant-retraction/l59.3mf` for the short-source
control (SHA256 `908612c123f73537d6cd07b3baaf93e5c48e97f84503030944c330ac3327f23d`).
Both archives here retain every other ZIP entry and change only six PA option
arrays inside `Metadata/project_settings.config`. No output is embedded.

Active High Flow selects source columns1/3 into logical slots0/1. Positive
model extrusion must be on logical tool1. Initial nozzle footer `[217,237]`,
self indices `[1,2]` and active High Flow confirm the unchanged variant premise.

Both contrasts use PA source `[0.011,0.022,0.033,0.044]` and bridge source
`[0.051,0.062,0.073,0.084]`, selecting `[0.022,0.044]` and `[0.062,0.084]`.
CSV model source strings have distinct values beginning0.01/0.02/0.03/0.04;
whole strings at columns1/3 must survive, including newline serialization.

- `distinct.3mf`: enable `[1,0,1,0]` selects `[0,0]`; adaptive `[0,1,1,0]`
  selects `[1,0]`; overhangs `[1,0,0,1]` selects `[0,1]`. No PA command
  should occur because selected enable flags are false.
- `static-enable.3mf`: enable `[0,0,0,1]` selects `[0,1]`; adaptive
  `[1,0,1,0]` selects `[0,0]`; overhangs `[1,0,0,1]` selects `[0,1]`.
  Existing Marlin command section must use selected PA0.044, not physical
  nozzle0, unselected source column1 or a model-fit value.

SHA256 identities are pinned in tests. Require stable Native first/replay,
successful public API/CLI, positive tool1 activity and raw API/CLI equality.
Compare all six complete footer lines and the complete existing PA override
command list. Retain complete-output comparisons separately; no general
whole-output parity is claimed.

Failure modes: cohort omitted, vectors shortened or reordered, wrong source
column, string CSV rows treated as columns, lost quoting/newlines, source-front
fallback replaced by default/zero, static-enable command consuming unselected
PA, unexpected adaptive activation, or API/CLI drift. The existing shared
gather nil/zero/nonzero witnesses remain unchanged.
