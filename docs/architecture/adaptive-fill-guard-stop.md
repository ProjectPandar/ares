# Adaptive fill guard STOP

The guard-removal proposal is closed with NO-RED STOP. Native-admitted inputs
cannot reach `adaptive.rs:42` or `:299` with an invalid spacing: upstream
`FillAdaptive.cpp:337-354` derives positive spacing from admitted density,
width, and multiline values; density<=0 surfaces are skipped at
`Fill/Fill.cpp:886,901`, width zero selects auto width at `FillAdaptive.cpp:305`,
and `fill_multiline` has a minimum of one (`PrintConfig.cpp:3000`). The zero
spacing path does not return an error: it avoids octree construction at
`PrintObject.cpp:1006-1007` and reaches the upstream null-dereference path at
`:1334-1336`. Anchoring swallows `InfillFailedException` at
`Fill/Fill.cpp:1495-1498`; `FillBase.cpp:108` has no offset error branch.
The existing guards remain unchanged.
