# Default installation resources for direct CLI slicing

Proposed rewrite boundary: OrcaSlicer `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`,
`src/OrcaSlicer.cpp:8169–8197` (installation root), `src/libslic3r/utils.cpp:296–306`
(resource setter/getter), and `src/libslic3r/PresetBundle.cpp:724–761` (use of
that root for system presets). Port installation discovery, not user preset
discovery or a new preset resolver.

## Installation layout

Native starts from the running program image (`OrcaSlicer.cpp:8167–8169`),
not the invocation string, `PATH` as a resource search list, or the working
directory. Let `exe` be the running Ares executable and `dir` its containing
directory. Resolve the root by the applicable Native branch only:

| Target | Default resource root | Native source |
| --- | --- | --- |
| Windows | `dir/resources` beside the installed executable | `OrcaSlicer.cpp:8176–8180`; `CMakeLists.txt:1290–1291` installs beside the executable. |
| macOS | `canonical(exe).parent().parent()/Resources`; `Contents/MacOS/ares` finds `Contents/Resources` | `OrcaSlicer.cpp:8172–8175`; case-sensitive spelling is `Resources`. |
| Linux, FHS package | The absolute resource directory fixed by that package at build time, in its architecture-independent data tree | `OrcaSlicer.cpp:8181–8184`; `CMakeLists.txt:114,1306–1309,1357` sets `${CMAKE_INSTALL_FULL_DATAROOTDIR}/OrcaSlicer` and configures `src/dev-utils/platform/unix/fhs.hpp.in:1–2`. An Ares FHS package supplies its own installed resource directory, not another application's path. This branch is not a search of `/usr/share` or a runtime prefix setting. |
| Linux, non-FHS package | `canonical(exe).parent().parent()/resources`; only if that path does not exist, try `canonical(exe).parent().parent().parent()/resources` for multiconfig builds | `OrcaSlicer.cpp:8185–8194`; `CMakeLists.txt:1334–1335` installs the portable `resources` tree under the prefix. |

For **each** branch `OrcaSlicer.cpp:8197` publishes the selected path in
`Slic3r::set_resources_dir`; `src/libslic3r/utils.cpp:296–306` stores and
returns it without further search, and `:2079–2083` joins `profiles` under
it for vendor-version lookup. Ares uses one CLI-local root, not a process
wide core global. On Linux the extra-parent branch is based on path absence,
not on missing vendor files, parse errors or missing presets in the primary
root. Neither macOS nor FHS has that fallback. Delete the unconditional
exe-adjacent default in `ares-cli/src/compatible_machine.rs`; retain exe
adjacency only on Windows, where it is Native behavior. Do not add an Ares
compatibility probe on Linux or macOS.

A supplied `--resources-dir` is the Ares CLI's existing explicit root
override. It wins outright, even if empty or invalid; the adapter must not
fall back to a different root or merge their contents. Update its help to
state the platform-specific default. Explicit paths may be relative to the
invocation directory as before; default discovery is independent of it.

## CLI boundary and preset supply

- At the base, `ares-cli/src/main.rs` calls `compatible_machine::load` only
  on the direct 3MF branch. It is the first consumer; resolve the root there
  after existing input/argument and route selection, and pass it to the
  existing reader for `profiles/BBL/machine_full/<original-printer>.json`.
  `OrcaSlicer.cpp:2820–2825` uses the same resource root and skips a missing
  individual file. Preserve upstream-result publication and diagnostic
  precedence, including invalid-value exits. Root discovery does not
  replace or decode the machine record by itself.
- When composed with the direct CLI system-catalog consumer, use
  the **same** resolved root for the existing JSON loader in
  `ares-cli/src/project_presets.rs` and pass its catalog to the existing
  core request. Neither the discovery branch on this base nor its replay
  should reimplement refresh, retained-value merge, slot admission or
  loader precedence. With usable installed resources, the no-flag direct
  route and the same explicit root use the same consumers and bytes,
  including moves and non-moves. `OrcaSlicer.cpp:2111–2117,2793–2817` owns
  lazy resolver creation and the slot refresh; `:3164–3200` owns retained
  printer/process lookup. Preserve the composed stack's direct eligibility
  for each, including any non-BBL retained-config handling it already has.
- Root selection is distinct from supplying a JSON catalog. An absent
  default root or a root without any supported JSON vendor manifests
  supplies no default catalog. Do not synthesize one or restore the
  removed missing-bundle fatal guard. An unresolved name retains embedded
  values under the existing direct contract. The machine-full reader can
  succeed even without a JSON vendor index. A malformed supplied bundle
  is not the same as an absent default; file/parse errors and explicit-root
  validation stay with the existing consumer, with no search elsewhere.
  Package staging must require parseable manifests and referenced records.
- Local discovery is for the direct 3MF route only. Do not turn defaults
  into a synthetic `--resources-dir` argument or mutate argument conflict
  rules. Preserve explicit `--project-preset-catalog`, `--load-settings`
  and `--load-filaments` routes and STL behavior; do not attach a default
  catalog to their requests. Core and WASM remain byte/handle APIs: no file
  I/O, executable probe, environment probe or new default supplier there.

Native `PresetBundle.cpp:731–735` first checks `data_dir()/system` for the
vendor, then `resources_dir()/profiles` (JSON or `.opc`). That **data-dir
precedence is out of scope**. In upstream `src/libslic3r/utils.cpp:365–412`,
`data_dir` is separate state and the Linux default can be a portable
`data_dir` or an XDG/HOME-derived location. Ares does not automatically read
any of those locations, including an installed copy of system presets there;
no HOME, XDG, `%APPDATA%`, CWD, test variable, or adjacent Native installation
participates in production discovery. Only caller-supplied explicit paths
can opt into such data. Do not read/write or synchronize user presets. The
upstream GUI installer at `utils.cpp:2086–2094` is not ported.

Native can ship only `.opc` (`PresetBundle.cpp:729–739`;
`utils.cpp:2066–2083`). Ares discovery does **not** add an OPC reader or
interpret the official Native AppImage as an Ares JSON package. Stage an
Ares-owned, pinned source-JSON `profiles/` tree with vendor manifests,
referenced preset files and required `machine_full` resources at the target
root. Record the source revision/tree in packaging and test evidence. A bare
`cargo build`/`cargo install` executable without staged resources is not a
resource-bearing installation; it may still use the explicit flag. No
network fetch or host profile installation at CLI startup.

## Test supply and acceptance

Build and stage a **private install** for public CLI tests. Copy the actual
binary into the platform layout and stage original, pinned source JSON at its
matching root. Use the source identity pinned for that comparison; for
2.5.0 Native compare against matching 2.5.0 source JSON. Keep this distinct
from both the Native official/seeded binary suppliers and existing explicit
catalog fixtures. `.github/actions/parity-profile-input/action.yml` already
shows how to pin and verify a source JSON tree for its beta lane; neither its
path variable nor that lane's beta identity becomes the runtime default.
Test tooling may use a verified input path to stage files, but the staged
CLI must discover them with **no** flag or runtime environment injection.
Never populate shared Cargo/debug or web/release target directories globally:
raw no-supply witnesses, explicit catalog legs, and WASM tests must not
silently start using installed profiles. If a host-CLI/browser byte oracle
needs a catalog, arrange equal explicit supplies on both sides or keep its
resource-free contract; a packaged CLI and a browser session do not share
an implicit supplier.

1. **Red before fix:** on Linux stage `<prefix>/bin/ares` and only
   `<prefix>/resources`. Derive a runtime `firmware-wipe.3mf` witness by
   changing only the embedded upward-machine list so it differs from the
   original machine resource. This matters because
   `UpwardMachineSource::select(FetchOriginal(None))` retains the embedded
   list. Require the no-flag subprocess to match the same staged Ares
   binary with an explicit root and the paired Native result. On the base
   it instead probes `<prefix>/bin/resources` and misses the installed
   bytes. Keep existing bare `cli_invalid_values_*` witnesses unchanged;
   the derived witness is additive, with move/no-move variants and checks
   for the expected result channel, stderr/stdout, and no output-file
   leakage on invalid input. After composition, add a successful, catalog-
   sensitive BBL witness whose resolved settings differ from genuine
   no-supply; the matching explicit-root control proves that discovery feeds
   both catalog and original-machine consumers.
2. **Layout and precedence controls:** cover Windows zip, macOS bundle,
   Linux portable and Linux multiconfig; for FHS, use a package build with a
   pinned private absolute data root and test it without relative lookup.
   Use same-named decoys in an unrelated CWD and private home/config tree,
   and on Unix in the unsupported exe-adjacent layout. Launch a portable
   fixture via a symlink from another directory where supported. Check
   Linux primary present but missing the requested data: do not consult the
   multiconfig location. Check distinct explicit override, explicit
   missing root, resource-free install, a single missing record,
   explicit catalog/load requests, non-BBL and STL controls. Assert these
   are actual executions, not skipped-supplier passes. Use native-target
   execution for unsupported host layouts before claiming package parity.
3. **Core-count invariance:** for identical input, arguments, executable,
   resource snapshot and producer metadata, varying CPU affinity must not
   affect the selected root, preset order or result. Resolve the root once
   in the CLI before slicing, not per worker; preserve deterministic catalog
   order. In a pinned installed-catalog success witness compare complete
   G-code and move CSV across one CPU and a multi-CPU affinity set, with separate output
   directories and equal snapshots. Keep the existing organic-affinity
   regression. Compare structured failure artifacts under the same CPU
   variation where applicable; normalize only already-authorized volatile
   fields. Different resource bundles are different inputs, not a CPU-count
   comparison.
4. **Paired gate after implementation:** run the affected CLI, API,
   standalone move-oracle and host-web inventory including existing plain
   and explicit-supplier controls in both seeded and official modes; record
   exact IDs, source/bundle identity, resolved layout, commands and
   artifacts. Require no dropped IDs and no P→F; any fixture supply change
   must be explicit and reviewed rather than masking a no-flag witness.
   Run fmt, workspace Clippy and the core/WASM target check. The spec-only
   change does not claim implementation or test results for these cases.
