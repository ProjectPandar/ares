# Slicing status callback

## Scope

This ports the slicing progress channel of `PrintBase` (v2.5.0-beta `345efec64`): the
`SlicingStatus` value, the registered status callback and the `set_status` calls that
`Print::process`, the support generators and `GCode::process_layers` make while a plate slices.
Three consumers use the same core callback:

- the web app shows the latest status while a plate slices;
- `ares-cli --progress` prints each status to stderr;
- native clients get the same callback from the core API.

The callback reports progress only. It never changes what the slice computes, so G-code bytes
are identical with and without a callback, and stay core-count invariant.

## Upstream boundary

| upstream | what is ported |
| --- | --- |
| `PrintBase.hpp:443-481` `SlicingStatus` | `percent`, `text`, `flags` with `DEFAULT` and `RELOAD_SCENE = 1 << 1` |
| `PrintBase.hpp:482-492` `status_callback_type`, `set_status_callback` | a borrowed `FnMut(&SlicingStatus)` registered on the slice context |
| `PrintBase.cpp:123-128` `PrintBase::set_status` | call the callback when one is registered; otherwise do nothing |
| `PrintObjectSlice.cpp:889` | `5`, `Slicing mesh`, `RELOAD_SCENE` |
| `PrintObject.cpp:528` | `15`, `Generating walls` |
| `PrintObject.cpp:631` | `25`, `Generating infill regions` |
| `PrintObject.cpp:819` | `35`, `Generating infill toolpath` |
| `PrintObject.cpp:877`, `:900-909` | `40`, `Z contouring`, then `40`, `Z contoured layer k/n (p%)` |
| `PrintObject.cpp:970` | `50`, `Checking support necessity` |
| `PrintObject.cpp:999` | `50`, `Generating support` |
| `TreeSupport.cpp:1914-1950`, `:2955` | `55`, `56` `Support: generate contact points`, `60`, one status per dropped layer, `65`, `70` |
| `TreeSupport3D.cpp:3622`, `:3700` | `60`, `69`, `Generating support` |
| `PrintObject.cpp:940` | `71`, `Detect overhangs for auto-lift` |
| `Print.cpp:3638` | `70`, `Generating skirt & brim` |
| `PrintObject.cpp:1061`, `:1079`, `:1097` | `75`, `Optimizing toolpath`, three times per object |
| `Print.cpp:3990` | `80`, `Generating G-code` |
| `GCode.cpp:5149`, `:5248` | `80`, `Generating G-code: layer N` |

The text is the upstream English source string. Formatted texts follow the upstream format:

- `Z contoured layer %d/%d (%d%%)`: `k` counts finished layers, `n` is the object's layer count
  and `p = int(k / n * 100)`. The loop runs over layers `1..n` (`PrintObject.cpp:899`,
  `blocked_range(1, m_layers.size())`), so the last status is `(n-1)/n`.
- `Generating G-code: layer %1%` with `N = index + 1` into `layers_to_print`. By layer, `N`
  counts the plate's merged layers. By object, `N` restarts at 1 for each printed instance
  (`GCode.cpp:5229-5248`).
- In the dropped-layer status (`TreeSupport.cpp:2955`), the percent is
  `60 + int(10 * (1 - float(layer_nr) / contact_nodes.size()))`. It fires only for layers that
  hold contact nodes, from the top layer down to layer 1.

### Order

Statuses fire in the order `Print::process` and `Print::export_gcode` fire them. Each per-object
status fires once per print object, in object order. Per plate:

1. `Slicing mesh` per object, then `Generating walls` per object (`Print.cpp:3420-3448`).
2. Per object, `Generating infill regions` then `Generating infill toolpath`. Upstream runs
   `prepare_infill(); infill();` in one object loop (`Print.cpp:3461-3481`).
3. `Z contouring` and its layer statuses, per object that needs contouring.
4. Support, per object: `Checking support necessity` when the object has no support (`has_support`
   is false; the CLI's `no_check` flag is false). Then `Generating support` when
   `(has_support && layers > 1) || (has_raft && layers > 0)`, followed by that generator's tree
   statuses.
5. `Detect overhangs for auto-lift` per object.
6. `Generating skirt & brim` once.
7. `Optimizing toolpath` three times per object (wall, infill and support paths).
8. `Generating G-code` once, then one `Generating G-code: layer N` per layer printed.

Native `generate_support_material` runs objects in a `tbb::parallel_for`, so native support
statuses of different objects can interleave in any order. Ares reports every object's necessity
check first, then each supported object's generation in object order, which is one of the orders
native can produce.

### Where Ares runs the work

Ares does not run every step at the point where `Print::process` runs it. The status still fires
at the upstream position, so the sequence a client sees equals OrcaSlicer's. Where the work runs
elsewhere:

- Wall and infill path simplification (`posSimplifyPath`, `posSimplifyInfill`) runs in
  `project_slice::path_simplification::apply`, before seam placement and before G-code export
  starts. Its two `Optimizing toolpath` statuses fire with the support-path one, just before
  `simplify_support_layers` in `gcode_emit::emit`.
- Auto-lift overhang detection runs inside `AutoLift::prepare` during export, and only for the
  `Auto` z-hop type. Ares skips the detection in other cases because its result is unused there.
  `Detect overhangs for auto-lift` fires for every object before skirt and brim, as native does.
- Ares prepares the infill of all objects, then fills all objects. The interleaved
  `25` / `35` pairs fire together, before infill preparation starts.

Moving these steps to their `Print::process` positions would change which data tool ordering and
the wipe tower see. That reorder is a separate pipeline slice, not progress work.

## Rust destination

### Core: `crates/ares-core`

- `SlicingStatus<'a> { percent: i32, text: &'a str, flags: u32 }`, with the associated
  constants `DEFAULT` and `RELOAD_SCENE`. Public, and re-exported from the crate root.
- `SlicerContext<'a>` holds `Option<&'a mut dyn FnMut(&SlicingStatus<'_>)>`. It plays the role
  of `PrintBase::m_status_callback` for one slice.
  - `pub fn set_status_callback(&mut self, callback: &'a mut dyn FnMut(&SlicingStatus<'_>))`.
  - `pub(crate) fn set_status(&mut self, percent: i32, text: fmt::Arguments<'_>, flags: u32)`.
    Without a callback this is one branch. A constant text is passed without allocating
    (`Arguments::as_str`); a formatted text is built only when a callback is registered.
- Every project-slice entry point already takes a `SlicerContext`. The stages that report
  statuses take `&mut SlicerContext`:
  - `perimeters`, for slicing and walls;
  - `slice_source`, for the infill pairs before `prepare_infill`, and for auto-lift detection and
    skirt and brim before G-code export;
  - `contour_z::prepare`;
  - `support_material::check_necessity` and `support_material::generate`, down to both tree
    generators;
  - `gcode_emit::emit` and the layer drive loop.
- The callback is not `Send`. No Ares stage that reports a status runs on another thread, and the
  web's JS function cannot be `Send`. A later parallel stage that reports progress decides then how
  its statuses reach the caller.
- Without a callback nothing is printed. Upstream's fallback logs at debug level, and the core
  has no logger.

### CLI: `crates/ares-cli`

- `--progress` installs a callback that writes `Percent {percent}: {text}` to stderr, one line per
  status. This is the text of `PrintBase::set_status`'s default output (`PrintBase.cpp:127`).
- It applies to every path that slices a project: 3MF projects, `--load-settings` and
  `--project-preset-catalog`.
- Without `--progress`, stdout, stderr and the output files are unchanged.

### WASM: `crates/ares-wasm`

The app slices through a plater session, so its two slice bindings take an optional JS function:
`PlaterSession::slice_processed` and `PlaterSession::slice_with_profiles_processed`. The adapter
calls the function with `(percent, text)` for every status. If the JS function throws, the slice
still completes, because cancellation is deferred.

### Worker package: `web/packages/ares-wasm-js`

- While a slice request runs, the worker posts `{ id, status: { percent, text } }` for each
  status, before the request's final response.
- `PlaterSession.slice` and `PlaterSession.sliceWithProfiles` take an optional `onStatus` callback,
  which receives those messages for that request.

### App: `web/app`

`slicePlates` stores the latest status in the slicing state.

- The status bar shows `Slicing plate {n}: {text}`, the prefix from `PartPlate.cpp:5476-5480`.
- A progress bar shows `percent`, as `Plater::priv::on_slicing_update`
  (`Plater.cpp:13145-13160`) sets `set_slicing_progress_percentage` from the latest status.

## Tests

Native-oracle E2E (`ARES_ORCA_BIN`, with no committed native outputs). Native runs with
`--debug 4 --logfile`, and the test reads its `default_status_callback` lines
(`OrcaSlicer.cpp:468-476`). Each line carries `percent`, `warning_step` and `message`; the test
keeps the lines with `warning_step=-1`.

1. **Core:** `crates/ares-core/tests/slicing_status.rs`. Each case slices one project through the
   public `slice_project_plate` with a recording callback. Then:
   - the recorded `(percent, text)` sequence must equal native's;
   - `RELOAD_SCENE` must be set on exactly the `Slicing mesh` statuses;
   - the G-code must equal the same slice's G-code without a callback.

   Cases, as 3MF variants written in the test:
   - `overhang-organic.3mf` as committed: organic tree, 3D generator statuses;
   - the same with `support_style = tree_hybrid`: `TreeSupport.cpp` statuses, including the
     per-layer drop statuses;
   - the same with `support_type = normal(manual)` and `support_style = grid`: normal support;
   - the same with `enable_support = 0`: `Checking support necessity`;
   - the same with `enable_support = 0` and `zaa_enabled = 1`: Z contouring;
   - `rotated-two-cubes.3mf` by layer, and with `print_sequence = by object`: two objects,
     infill interleaving and per-object layer numbering.
2. **CLI:** `crates/ares-cli/tests/cli_progress.rs` slices `overhang-organic.3mf` with and without
   `--progress`. Then:
   - stderr with the flag holds the core's statuses (checked against native by the core test) as
     `Percent N: text` lines, in order;
   - removing those lines leaves exactly the stderr of the run without the flag;
   - the G-code files are equal.
3. **Web:** `web/e2e/slicingProgress.spec.ts` slices a plate in the app. The texts the status bar
   shows must appear, in order, in the core's sequence. They must include `Slicing plate 1:
   Slicing mesh` and a `Generating G-code: layer N`, and the progress bar must follow the shown
   percent. Screenshots go in `target/web-e2e/slicing-progress`.

## Deferred

- The warning fields `warning_step`, `warning_object_id`, `message_type` and `warning_level`, and
  `status_update_warnings` (`PrintBase.cpp:131-155`). Slice warnings keep their current channel.
- `cancel_callback` and `throw_if_canceled`.
- The native CLI's `--pipe` JSON progress channel: `cli_callback_mgr_t`, `cli_status_callback`
  (`OrcaSlicer.cpp:456-466`), installed at `:7059-7085`, including its `4`, `Slicing begins`
  status.
- `RELOAD_SCENE`'s GUI consumer, which reloads the object shells (`Plater.cpp:13164-13172`). The
  flag is ported; the web does not act on it.
- The `Exporting G-code to <path>` text of `Print::export_gcode`. Native uses it only when called
  without a processor result; both the GUI and the CLI pass one (`OrcaSlicer.cpp:7192`).
- Objects that native slices once and shares (`need_slicing_objects`, `is_print_object_the_same`,
  `Print.cpp:3309-3380`). Native reports no per-object status for a sharing object; Ares slices it
  and reports its statuses.
- A status callback on the free WASM slice bindings (`sliceProjectPlateProcessed`,
  `sliceWithProfilesProcessed` and the unprocessed ones). The app uses them only without a plater
  session.
- SLA statuses (`SLAPrint.cpp`), and the STL `--options` path (`slice_stl_validated`), which is not
  a `Print::process` port and reports nothing.
