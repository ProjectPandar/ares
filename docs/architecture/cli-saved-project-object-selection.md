# Saved 3MF object selection and assignment options

Source pin: OrcaSlicer `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

| Option | `src/libslic3r/PrintConfig.cpp` | Saved project behavior |
| --- | --- | --- |
| `--skip-objects` | `:13084–13088` | Exclude loaded instances by their saved `identify_id`, not the 3MF resource ID or the runtime object ID. |
| `--clone-objects` | `:13090–13094` | Requires one count per input and a nonempty `--load-filaments` list, then rejects 3MF input. |
| `--load-filament-ids` | `:13191–13195` | Requires one ID per input and a nonempty `--load-filaments` list, then rejects 3MF input. |

All three are signed, comma-separated integer vectors; repeated occurrences append
(`Config.cpp:1865–1898`, `Config.hpp:1116–1137`). Clone validation precedes
filament-ID validation, and within each, the count check precedes the filament-list
check (`OrcaSlicer.cpp:1671–1706`). The file existence check follows these
prerequisites (`:1724–1729`). For an existing first 3MF, the assignment options
are rejected before archive or external filament preset ingestion (`:1742–1748`).
Each rejection is `CLI_INVALID_PARAMS` (`Utils.hpp:44`, POSIX 254), plate index
0, reason `Invalid parameters to the slicer.` (`OrcaSlicer.cpp:196`), no G-code.
Use the stage-specific diagnostic from the same source branch.

`Format/bbs_3mf.cpp:4739–4744,4996–5004,2465–2498` supplies the saved instance
identity (`Model.hpp:1435`). Reuse the existing `ProjectInstance` import state.
`OrcaSlicer.cpp:1383–1385,1650–1656` builds the skip lookup; `:6558–6589,6611–6618,6666–6674`
first checks the plate's print volume, then marks matching instances as
non-printable before per-instance checks and `Print::apply`. Apply this at the
selected-plate boundary in `ares-core` after the existing load, model actions,
plate selection, and initial print-volume check. Preserve unmatched instances
and their identities. An unknown ID does not affect output; if all inside
instances are skipped, return `CLI_NO_SUITABLE_OBJECTS_AFTER_SKIP`
(`Utils.hpp:81`, POSIX 196), the selected one-based plate index, and reason
`No printable objects to slice after skipping.` (`OrcaSlicer.cpp:230`). Do not
collapse that failure into the earlier empty/outside-plate error.

Keep the CLI responsible for argv, file I/O and failure reporting, and the
option checks and printable-state changes in core. The public seam is
`ares slice -o FILE [--plate N] PROJECT.3mf`. Use a synthetic saved 3MF with two
objects whose resource IDs differ from their identify IDs, in both Native
modes with live runs and replays. Compare whole ordered G-code using the
existing generator normalization and `result.json` using the existing elapsed
time normalization. For assignment rejections compare the diagnostic, status,
and report, and assert that no G-code was produced.

Deferred: standalone STL and other non-project ingestion, positive cloning
and filament assignment (`OrcaSlicer.cpp:1924–1979`), duplicate-dependent
placement, and updating project-export thumbnails or skip annotations.
