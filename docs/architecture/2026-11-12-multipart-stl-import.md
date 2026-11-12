# Multiple STL files and multipart model loading

## Source and scope

This slice rewrites `libslic3r::Model::convert_multipart_object` and the
multi-volume branch of `ModelObject::split` for a batch of STL inputs. The
calling policy comes from `Plater::add_model` → `Plater::priv::load_files` and
`load_model_objects`. It extends the existing plater/session model; it does
not introduce another slicing pipeline.

The source is OrcaSlicer v2.5.0,
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`:

| Source | Included behavior | Rust destination |
| --- | --- | --- |
| [`Model.cpp:926–977`](https://github.com/OrcaSlicer/OrcaSlicer/blob/40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19/src/libslic3r/Model.cpp#L926-L977), `Model::convert_multipart_object`; `Model.hpp` model/object/volume/instance data | Preserve input order and volume geometry; restore the common coordinate frame; form one object with its parts. | `ares-core/src/project/domain/multipart.rs`, with the existing STL/profile construction in `project/domain/profile_input/stl.rs`. |
| [`Model.cpp:2220–2346`](https://github.com/OrcaSlicer/OrcaSlicer/blob/40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19/src/libslic3r/Model.cpp#L2220-L2346), `ModelObject::split` | For the multipart batch, split by model-part volume, carry its placement to the new instance, clear the volume offset and retain each part's name. | The same core multipart module; reuse existing object, volume and instance types. |
| [`Format/STL.cpp`](https://github.com/OrcaSlicer/OrcaSlicer/blob/40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19/src/libslic3r/Format/STL.cpp), `load_stl` | One mesh volume per STL; retain filename identity. | Existing `load_model` / STL mesh loading, called once per input during a batch load. |
| [`Plater.cpp:16670–16692`](https://github.com/OrcaSlicer/OrcaSlicer/blob/40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19/src/slic3r/GUI/Plater.cpp#L16670-L16692), `Plater::add_model` | Classify multiple non-3MF inputs as `MultipleOther`, then call `load_files` with `ask_multi`. | `ares-core/src/plater/stl_files.rs` receives the resolved choices; the web adapter asks the question. |
| [`Plater.cpp:10075–10125`](https://github.com/OrcaSlicer/OrcaSlicer/blob/40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19/src/slic3r/GUI/Plater.cpp#L10075-L10125), `Plater::priv::load_files` | Collect models, combine when Yes or Auto-Drop is off, then load; No with Auto-Drop off additionally splits the combined object. | `ares-core/src/plater/stl_files.rs`. |
| [`Plater.cpp:10316–10403`](https://github.com/OrcaSlicer/OrcaSlicer/blob/40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19/src/slic3r/GUI/Plater.cpp#L10316-L10403), `load_model_objects`; [`10983–11039`](https://github.com/OrcaSlicer/OrcaSlicer/blob/40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19/src/slic3r/GUI/Plater.cpp#L10983-L11039), `split_object` | Empty-plate load placement; ground drop or the negative-Z clamp; retain placement when splitting with Auto-Drop off. | Core plater load/placement using the selected printer's bed and existing plate list. |

Only ordinary STL model-part inputs are added here. The source accepts other
formats and model states too; those adjacent paths are not implied by this
interface.

## Model and placement

An **object** owns one or more **volumes** (parts). An **instance** places an
object on a plate. Combining files must not concatenate their triangles into
one anonymous mesh or merely combine their browser scene entries.

The batch keeps the selected file order. Each STL contributes one volume,
including when its mesh has disconnected solids. The combined object uses the
first input's filename stem as its object name; its volumes retain the source
object names. There is no filename-based material assignment. Upstream's
`max_extruders` argument is unused in this conversion: its operative priority
is volume extruder → object extruder → default. This slice does not add
round-robin extruders, filament slots or nozzle inference.

The conversion reverses each source object's `origin_translation` and composes
instance and volume transforms to a common frame. The fresh-STL path may retain
that frame directly rather than introducing centering state solely to undo it.
Its resulting geometry, volume/instance transforms and bounds must match the
source operation. Fresh STL inputs have no archive-provided modifiers,
painting, custom object config or extra instances.

For N valid STL inputs, with N at least two:

| Answer | Auto-Drop | Result |
| --- | --- | --- |
| Yes | On | One object, N volumes; center the whole object on the empty bed and drop its lowest point to Z=0. Relative part placement is retained. |
| Yes | Off | One object, N volumes; center its XY placement, preserve its original Z placement unless the lowest point is below the bed, then lift the whole object just enough. |
| No | On | N objects, one volume each; use the existing ordinary object load placement and drop each object independently. Do not promise retention of their relative source positions. |
| No | Off | Convert to one multipart object, load it without dropping, then split by volume without dropping. N objects retain the combined object's relative XYZ placement. |

Auto-Drop off does not mean that negative Z is retained. `load_model_objects`
lifts by `-min(min_z, 0)`. For No/Off this happens to the combined object before
splitting, so one common lift preserves the parts' relative Z difference.
Do not independently center or ground the parts afterward.

Use the existing empty-plate positioning behavior. Automatic arrangement,
nearest-empty-cell placement under the optional `AUTOPLACEMENT_ON_LOAD` build
branch, oversized-model dialogs and unit-conversion questions are outside this
slice. There is no new arrangement algorithm or repair of overlapping models.

## Core interface and slicing

The new batch entry accepts named STL byte inputs and the two choices,
`as_multipart` and `auto_drop`. Profile-aware loading uses the existing explicit
catalog, profile selection and option-override inputs. The core neither asks a
question nor reads files, environment variables, browser state or resources.

A focused `plater/stl_files.rs` module owns batch loading and its placement
policy. Multipart model operations live with the project domain types, not in
WASM, JavaScript or a viewer module. Expose `PlaterSession::load_stl_files` and
`PlaterSession::load_stl_files_with_profiles`; their result is an ordinary
`PlaterSession`, whose project and scene expose the object/volume structure.

The current session retains a single `Vec<u8>` and both session slicing entries
reload it. That is a temporary single-input compatibility shell, not a model
representation suitable for a batch. It must be replaced for the batch path:

- Keep the assembled model as the batch's original, in-memory slicing source.
  Use an explicit model source alongside byte sources at the existing
  `project_slice::ProjectSource` loading seam; do not substitute empty bytes,
  the first file's bytes or a generated 3MF archive.
- Keep this original model distinct from the session's edited state. Share mesh
  storage using the existing project mesh ownership. Apply session edits once;
  copying an instance must not duplicate it a second time during slicing.
- Extend the existing profile load/preparation seam to compose settings onto
  that STL model. Reuse the current profile rules and override ordering; do not
  duplicate the settings parser or rewrite the 3MF preset-refresh lifecycle.
- Both session slicing entries must consume every batch volume through the
  existing `Print` preparation and processing path. The scene and sliced model
  must describe the same geometry after placement and subsequent session edits.
- Retain explicit byte loading for existing single-file callers. Neither source
  kind silently retries the other kind.

Keep new Rust modules within 400 physical lines. Split the touched session or
profile-entry module only as needed for this interface, not as unrelated
cleanup. No new dependencies, unsafe code or mutable global state are needed.

## WASM and web adapters

Expose the batch load through `ares-wasm` and `ares-wasm-js`, including worker
request/response transport. Retain all named inputs and both choices through
profile-dependent session creation. Use an explicit batch source in TypeScript
so a single-file slice cannot accidentally receive only the first STL.
Geometry conversion and placement remain core operations.

The browser file pickers in `TopBar` and `PlaterToolbar`, and the `Shell`
drag/drop entry, pass all selected files to the same import action. More than
one STL opens the existing-theme modal with:

- Title: **An object with multiple parts was detected**
- Question: **Load these files as a single object with multiple parts?**
- Checkbox: **Auto-Drop**, checked initially for each import.
- Answers: **Yes** and **No**.

Use the existing dialog, theme, typography and button conventions; this is a
local addition to the Prepare workflow, not a visual redesign. The checkbox has
an accessible label, focus stays in the modal, and the answers are keyboard
operable. Closing without an answer leaves the current project unchanged.

A successful batch import **replaces** the loaded project, as the existing web
single-file import does. It does not append to an existing model. Keep the old
project intact while reading the files and until the replacement loads
successfully; a malformed later STL must not leave a partially imported batch.

One STL or one 3MF keeps its existing import path and does not gain this dialog.
Multiple 3MFs and mixed STL/3MF selections are not supported by this entry and
produce a clear input error rather than silently choosing a subset.

## Native source identity

The first Native model observations use the beta chain-probe environment at
`345efec64c7b95a05ce5cf96df0dde66a158835c`, selected explicitly through
`parity-250b.env` and `nonparity-oracle-250b.env`. The following complete source
files, including EOF, are identical between that revision and the v2.5.0 source:

| File | Git blob at both revisions |
| --- | --- |
| `src/libslic3r/Model.cpp` | `c4cff42a64b628c5a3b417beeeae6cf3624da568` |
| `src/libslic3r/Model.hpp` | `6d9797ff675b329c32554ad09ca8ab3ebbfbac4e` |
| `src/libslic3r/Format/STL.cpp` | `83510043a234805183c521af01cf9396c7310b1e` |
| `src/libslic3r/Format/STL.hpp` | `e545e1c6d013afaa7f2ca25db40356dd4a7537ee` |

These same bytes were checked against the merged beta producer's source tree.
Bind each runtime observation to its actual binary, source tree, invocation,
inputs and outputs. Source equality does not turn the beta binary into a
v2.5.0 binary or establish whole-program equivalence.

`Plater.cpp` is different: its beta blob is
`e8df816036f80a7f1389c35f84650eec784cbc9f`; the v2.5.0 blob is
`db40f20cdbef6cc0a2ee761c5d9ae033f028f1b0`. Port its calling and placement policy
from v2.5.0. Native model-method observations are not an execution of that GUI
load path. Repeat the model/runtime comparisons in the subsequent batch using
the v2.5.0 Native environment. Do not change global source pins or relabel any
existing beta results.

## Tests and repeatable outputs

Write the failing end-to-end checks before implementation. Use original STL
bytes as the inputs to both implementations, not an expected-output archive or
committed golden. A missing new Rust interface is a compilation failure, not a
runtime Native comparison failure; separately run the browser check against the
existing import behavior.

The Core/Native checks exercise all four choice combinations on at least two
asymmetric STLs with distinct original XYZ positions. Compare ordered object
and volume names, counts, mesh geometry, volume and instance transforms, and
world bounds. Include positive overall minZ and negative overall minZ cases so
an unconditional ground drop or an independent per-part clamp cannot pass.
For No/Off, verify both relative XY and relative Z after splitting.

Also check the complete assembled model reaches slicing, including the second
file, and that a subsequent session move/instance-copy edit is applied once.
An input error must not return a partial batch. Single-file 3MF/STL behavior
continues to pass its existing checks.

Playwright on port **4184**, never 4174, must cover:

1. Select two STLs, see the exact question and checked Auto-Drop, answer Yes,
   observe one object with two parts, and slice successfully.
2. Select the same two STLs, answer No, observe two objects, and slice
   successfully.
3. Load through drag/drop and the alternate picker; verify the choice reaches
   the same core entry. Exercise Auto-Drop off and retained relative Z.
4. Import over a loaded project and confirm replacement, not append; reject a
   malformed later STL or mixed-format batch without losing that project.

Each runtime attempt writes its input hashes, Native model observation,
structured Rust/browser model results, comparator outcome, logs and sliced
G-code. Browser attempts also retain screenshots and the exact served WASM/JS
identity. Do not add comparison masks or commit generated outputs.

Run formatting, strict locked workspace/all-target Clippy, Rust checks through
`cargo nextest run`, and core plus WASM-adapter `wasm32-unknown-unknown` checks.
Browser execution is separate from WASM compilation. The model logic remains
memory-only and portable to WASM browsers, Windows, macOS and Linux.

## Adjacent behavior not included

- `Model::looks_like_multipart_object` at `Model.cpp:898–915`, and the
  single-file question at `Plater.cpp:9957–9959`: multiple objects from one
  archive are a different entry. Do not add an unused heuristic just to name it.
- Appending imports to an existing project; mixed-format or multiple-project
  import; OBJ, STEP and other formats.
- General conversion of painted/configured archive objects, arbitrary existing
  instances, modifiers, text volumes, CAD recipes or painting remaps; the
  single-volume connected-component branch of `ModelObject::split`.
- A general object-list merge/split command, undo history, assembly-view editing,
  profile discovery, filament-slot changes or new native build variants.
- Global upstream-baseline migration, CI pin cleanup and Native environment
  construction. Those remain separate from this model import slice.
