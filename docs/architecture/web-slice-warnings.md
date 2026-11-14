# Browser slicing warnings

The core's empty-layer warning (`GCode.cpp:2404–2467`) is a successful-slice
warning. The browser must expose it without replacing the core warning producer
or adopting the CLI's refusal policy.

## Upstream boundary

Source revision: `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.
Line references below are relative to that revision's `src/` directory.

This slice carries `libslic3r::PrintStateBase::Warning` from the successful
processed-slice result through WASM to a browser presentation of the existing
`Plater::priv` / `NotificationManager` warning behavior. It does not introduce
an Ares-owned warning detector or slicing pipeline. `ProcessedGCode.warnings`
is the existing compatibility representation of the upstream warning list;
`SliceOutput` and the worker response are adapters, not warning authorities.

### Production, completion, and display

| Upstream source | Behavior retained at the Rust/browser boundary |
| --- | --- |
| `libslic3r/PrintBase.hpp:60–101` | `WarningLevel` distinguishes `NON_CRITICAL` and `CRITICAL`; a warning has a current-state flag, UTF-8 message, and message ID. A warning is not a slicing exception. |
| `libslic3r/PrintBase.hpp:439–483,599–606` | Warning status identifies its owning `Print` or `PrintObject`, its step, type, and level. Adding an active-step warning notifies the status callback. The owner ID is separate from the message. |
| `libslic3r/GCode.cpp:2457–2467` | Empty extrusion-layer ranges produce the complete object/mesh message and a `CRITICAL`, `SlicingEmptyGcodeLayers` warning through the **Print-level** active-step method; this call does not throw. |
| `slic3r/GUI/PartPlate.cpp:5461–5477` | The active plate installs the print status callback, queues `EVT_SLICING_UPDATE`, and prefixes only ordinary progress text with the plate number. |
| `slic3r/GUI/BackgroundSlicingProcess.cpp:260–320` | Fresh FFF processing calls `Print::process`, queues slicing-completed, then exports temporary G-code. Slicing-completed alone is not the final successful result: G-code export can still emit warnings or fail. |
| `slic3r/GUI/BackgroundSlicingProcess.cpp:308–321,400–418,496–514` | Finalization/export is distinct from exception capture and the final Finished/Cancelled/Error event. Ordinary completion retains successful G-code rather than treating a warning level as an exception. |
| `slic3r/GUI/Plater.cpp:7913–7918,8233–8234,13150–13228` | Event handlers retrieve the affected step's current warnings and push notifications. Default presentation is `WarningNotificationLevel`; the `SlicingReplaceInitEmptyLayers` and `SlicingEmptyGcodeLayers` event types select `SeriousWarningNotificationLevel`. This is a **type-specific** presentation rule, not a claim that every CRITICAL warning maps to that notification enum. |
| `slic3r/GUI/Plater.cpp:13231–13251,13346–13405,13442–13447` | Slicing-completed updates scene shells. Final processing handles errors and updates slice-result validity separately. Warning presence does not invalidate a successful result. |
| `slic3r/GUI/NotificationManager.cpp:2070–2096` | A slicing-warning notification retains text, owner ID, step, and message ID; when a model object is available it offers a selection link. |
| `slic3r/GUI/NotificationManager.cpp:439–443` | Serious warnings receive distinct warning styling; they are not automatically promoted to slicing errors. |
| `slic3r/GUI/Plater.cpp:13273–13323` | Starting a new slice clears current warnings; invalidation actualizes/grays notifications and removes released owners. Warnings belong to a slice/object lifecycle, not an application-global log. |

### Export is not prohibited by CRITICAL warnings

The comment on `PrintStateBase::Warning` (`PrintBase.hpp:89`) describes an
export-time modal so warnings cannot be missed. The concrete pinned GUI behavior
is more specific:

- `Plater.cpp:13254–13257`: `on_export_began` invokes `warnings_dialog()` when
  `show_warning_dialog` is set; it **does not use the return value**.
- `Plater.cpp:13325–13342`: that dialog includes the first line of **all** current
  warnings, not just CRITICAL ones. It has `wxOK`, not a reject-export choice.
- `BackgroundSlicingProcess.cpp:308–321`: the background worker queues export-began
  and continues finalization/upload. The event is not a warning-based export gate.

Therefore the browser must not disable G-code export, reject the slice promise,
clear G-code, or mark the result failed because a warning is CRITICAL. It must
make the warning visible. Any export-time acknowledgement is informational, not
a second validity check. The CLI's separate tag-3 refusal policy
(`OrcaSlicer.cpp:7141–7172`, owned by `ares-cli`) must **not** be copied into WASM.

## Warning data contract

The browser-facing warning record is:

```ts
interface SlicingWarning {
  level: "nonCritical" | "critical";
  message: string;
  objectId: string | null;
}
```

- `level` is an exhaustive mapping from the core `WarningLevel`, not inferred
  from wording, message type, or the success/error state of the request.
- `message` preserves the producer's complete UTF-8 text, newlines, object name,
  punctuation, and numerical formatting. Do not truncate it to a status label,
  parse it for IDs, reformat its ranges, or independently regenerate the warning.
- `objectId` is an opaque decimal owner ID when genuine upstream-equivalent
  owner provenance is available. It is a `Print`/`PrintObject` warning owner,
  **not** a zero-based scene index or necessarily a selectable model-object ID
  (`PrintBase.hpp:450–456,473–475`). String transport avoids imposing JavaScript
  Number precision on native `size_t` IDs (`ObjectID.hpp:22–39`). `null` means that this warning's core
  representation did not retain an owner ID; it does not mean object index zero.
  The core `PrintWarning` representation does not retain an owner ID. Its
  transport uses `null` for that missing provenance; it must not fabricate an ID
  from the object name or expose an internal traversal index. Do not add a new
  ID allocator for this slice. Object-selection links require separately retained
  model-object provenance and are deferred.

Every covered processed result contains its final warning array, including an
explicit empty array when the observed list is empty. Preserve source order and
all emitted records; do not filter out ordinary warnings or merge records by
message text. Message-ID/step/current-state notification reconciliation remains
upstream state-management work, not a new browser heuristic.

The proposed WASM byte boundary is `SliceOutput.takeWarnings()`: UTF-8 JSON bytes
containing the array above, consumed like `takePrintStatistics()`. Capture warnings
from `ProcessedGCode` before its fields are moved into `SliceOutput`; read them in
`processedResult` before `output.free()`. The JS processed result carries the
parsed `warnings` array through the existing worker request/response. JSON records
are structured-cloned normally; they are not mixed into the typed geometry buffers.
No G-code comments, stderr scraping, CLI result.json parsing, or second slice is a
warning transport.

## Destination boundaries

At the current `origin/main` base, warning data is lost at
`crates/ares-wasm/src/moves.rs::SliceOutput::from_processed`.
`crates/ares-core/src/gcode_processor.rs::ProcessedGCode` already owns warnings;
`crates/ares-core/src/print/warnings.rs::PrintWarning` owns their meaning.
Core owns the empty-layer warning (`GCode.cpp:2404–2467`); `ares-cli` owns the
CLI refusal policy. Transport consumes the core result without reproducing
or modifying either policy.

Transport ownership is bounded to:

1. `ares-wasm`'s `SliceOutput` warning serialization, preferably in a dedicated
   warning adapter module; do not grow `moves.rs` beyond the repository LOC limit.
2. `web/packages/ares-wasm-js/src/worker.ts::processedResult` and its processed
   response types: carry the warning array for processed project/profile slices
   and both ordinary/profile `PlaterSession` slice entrypoints. Do not put slicer
   logic in `ares-wasm-js` or `ares-vgcode`.
3. `web/app/src/state/actions.ts::setSliceResult`, `state/store.ts::SliceState`
   and `plateSlices`: retain warnings with the exact successful result and plate.
4. A slicing-notification surface mounted by `web/app/src/shell/Shell.tsx`, separate
   from `LoadWarning`, plus the existing export actions' informational warning
   presentation. Import/config substitutions remain `loadWarning`, not slice warnings.

Byte-only Rust/WASM APIs and the JS `sliceStl` worker route currently bypass
`SliceOutput`. Their warning-result redesign is **not** covered. Do not advertise
warning parity for them or synthesize `[]` for an unobserved warning list. Keep the
processed warning-bearing contract explicit in response types; any type adjustment
needed to distinguish the byte-only result must preserve this distinction, not use
an optional field as a legacy fallback. The interactive project/profile/plater
routes are the acceptance boundary. The existing bare-STL pipeline is only a
temporary compatibility shell, not a pipeline to extend in this milestone.

## Browser presentation and lifecycle

The selected surface is **persistent slicing notifications**, not replacement of
progress/completion text in `shell/StatusBar.tsx`.

- Mount notices inside the existing workspace notification area, beside but
  separate from the import/config warning. Follow incumbent notification styling
  and controls. A CRITICAL record is labeled as a critical slicing warning with
  an alert announcement; ordinary warnings remain visibly labeled warnings.
  Severity must not depend on color alone or be labeled a failed slice.
- Show every full message, preserving line breaks. Long/multiple warnings must
  remain scrollable/readable without obscuring export controls. No transient
  timeout. Announce the completed batch once rather than every render.
- Retain warnings in the completed per-plate result. Switching plates shows that
  plate's warnings; Slice all must not discard earlier plates' records. Closing a
  notice affects its presentation only, not stored warnings or G-code validity.
- Starting a replacement slice removes the previous visible warnings for that
  attempt. On success replace the snapshot; a warning-free success clears them.
  Model/config invalidation clears the corresponding completed-result snapshots
  using the existing invalidation path. An actual error keeps the existing error
  behavior and must not present another plate's old warning list as current.
- Keep Export enabled for a valid completed result regardless of severity. At
  export, present an informational acknowledgement of that result's warnings
  (all severities, matching the pinned GUI's concrete behavior), then allow the
  unchanged G-code download. Closing an earlier notice must not erase this export
  information. Export all associates each warning with its selected plate and
  does not silently omit warned plates. Do not add a “fix first” or reject-export
  policy. The browser need not reproduce the native worker/UI event race.

## Included and deferred

Included: completed FFF processed-result warning transport, ordinary and CRITICAL
severity, exact messages, explicit owner-ID availability, per-plate result lifetime,
persistent browser notices, and non-prohibiting informational export handling.

Deferred: new warning detectors or mesh repairs; changes to `ares-cli`'s refusal
policy; byte-only API redesign; live intermediate warning
streaming; warning snapshots on failed exports; PrintState step invalidation/gray
reconciliation and message-ID deduplication; model-object navigation and newly
captured owner IDs; SLA, printer upload/send flows, localization infrastructure,
and a general-purpose notification-center redesign. No parity-env, oracle-pointer,
serializer-probe, mesh-fixture, comparator, or golden change is part of this slice.

## Tests

- **Internal-gap mesh:** two closed boxes in one object, separated vertically by
  an internal gap larger than the configured layer height and support clearance,
  with supports disabled. The pinned native collector must emit
  `CRITICAL SlicingEmptyGcodeLayers`. Every covered processed entrypoint preserves
  its complete message and `critical` level, reports `objectId: null` when core
  provenance is absent, and returns successful nonempty G-code. The visible
  notification does not turn the result into `error` or disable Export.
- **Cantilever mesh:** a horizontal overhang on a vertical stem, supports disabled,
  with the pinned configuration producing `NON_CRITICAL SlicingNeedSupportOn`.
  Its ordinary warning survives transport without severity promotion. Combining
  the cantilever and internal-gap objects preserves all emitted records in order.
- **Closed cube:** a manifold 10 mm cube under a warning-free configuration yields
  `warnings: []`. Replacing a warned result with this result clears the visible
  notifications without changing G-code or preview buffers in transport.
- **Warning-record fixtures:** multiple multiline UTF-8 messages, a null owner,
  and an available owner ID `"9007199254740993"` retain exact text, order and ID
  precision across JSON extraction, worker response and component state. Reading
  warnings before freeing `SliceOutput` retains the complete array.
- **Two-plate project:** an internal-gap object on one plate and a closed cube on
  another keep distinct warning snapshots through Slice all and plate switching.
  Closing a notice preserves its stored warning and export-time informational
  acknowledgement. Export and Export all retain every selected plate's G-code
  bytes and do not refuse a plate because it has a critical warning.
- **Invalidation and failure:** config/model changes invalidate the corresponding
  completed warning snapshot. A truncated STL fails slicing and cannot show a
  previous plate's warnings as current. Progress/completion text and import/config
  notices remain separate from the completed slicing-warning notifications.
